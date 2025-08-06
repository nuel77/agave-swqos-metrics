use itertools::Itertools;
use quinn::{VarInt, VarIntBoundsExceeded};
use solana_quic_definitions::{
    QUIC_MAX_STAKED_CONCURRENT_STREAMS, QUIC_MAX_STAKED_RECEIVE_WINDOW_RATIO,
    QUIC_MAX_UNSTAKED_CONCURRENT_STREAMS, QUIC_MIN_STAKED_CONCURRENT_STREAMS,
    QUIC_MIN_STAKED_RECEIVE_WINDOW_RATIO, QUIC_TOTAL_STAKED_CONCURRENT_STREAMS,
    QUIC_UNSTAKED_RECEIVE_WINDOW_RATIO,
};
use solana_sdk::packet::PACKET_DATA_SIZE;
use solana_sdk::pubkey::Pubkey;
use solana_streamer::nonblocking::quic::ConnectionPeerType;
use std::collections::HashMap;
use std::sync::Arc;

pub fn compute_max_allowed_uni_streams(peer_type: ConnectionPeerType, total_stake: u64) -> usize {
    match peer_type {
        ConnectionPeerType::Staked(peer_stake) => {
            // No checked math for f64 type. So let's explicitly check for 0 here
            if total_stake == 0 || peer_stake > total_stake {
                println!(
                    "Invalid stake values: peer_stake: {:?}, total_stake: {:?}",
                    peer_stake, total_stake,
                );

                QUIC_MIN_STAKED_CONCURRENT_STREAMS
            } else {
                let delta = (QUIC_TOTAL_STAKED_CONCURRENT_STREAMS
                    - QUIC_MIN_STAKED_CONCURRENT_STREAMS) as f64;

                (((peer_stake as f64 / total_stake as f64) * delta) as usize
                    + QUIC_MIN_STAKED_CONCURRENT_STREAMS)
                    .clamp(
                        QUIC_MIN_STAKED_CONCURRENT_STREAMS,
                        QUIC_MAX_STAKED_CONCURRENT_STREAMS,
                    )
            }
        }
        ConnectionPeerType::Unstaked => QUIC_MAX_UNSTAKED_CONCURRENT_STREAMS,
    }
}

pub fn calculate_stake_stats(
    stakes: &Arc<HashMap<Pubkey, u64>>,
    overrides: &HashMap<Pubkey, u64>,
) -> (u64, u64, u64) {
    let values = stakes
        .iter()
        .filter(|(pubkey, _)| !overrides.contains_key(pubkey))
        .map(|(_, &stake)| stake)
        .chain(overrides.values().copied())
        .filter(|&stake| stake > 0);
    let total_stake = values.clone().sum();
    let (min_stake, max_stake) = values.minmax().into_option().unwrap_or_default();
    (total_stake, min_stake, max_stake)
}

pub fn compute_recieve_window(
    max_stake: u64,
    min_stake: u64,
    peer_type: ConnectionPeerType,
) -> Result<VarInt, VarIntBoundsExceeded> {
    match peer_type {
        ConnectionPeerType::Unstaked => {
            VarInt::from_u64(PACKET_DATA_SIZE as u64 * QUIC_UNSTAKED_RECEIVE_WINDOW_RATIO)
        }
        ConnectionPeerType::Staked(peer_stake) => {
            let ratio =
                compute_receive_window_ratio_for_staked_node(max_stake, min_stake, peer_stake);
            VarInt::from_u64(PACKET_DATA_SIZE as u64 * ratio)
        }
    }
}

/// Calculate the ratio for per connection receive window from a staked peer
pub fn compute_receive_window_ratio_for_staked_node(max_stake: u64, min_stake: u64, stake: u64) -> u64 {
    // Testing shows the maximum througput from a connection is achieved at receive_window =
    // PACKET_DATA_SIZE * 10. Beyond that, there is not much gain. We linearly map the
    // stake to the ratio range from QUIC_MIN_STAKED_RECEIVE_WINDOW_RATIO to
    // QUIC_MAX_STAKED_RECEIVE_WINDOW_RATIO. Where the linear algebra of finding the ratio 'r'
    // for stake 's' is,
    // r(s) = a * s + b. Given the max_stake, min_stake, max_ratio, min_ratio, we can find
    // a and b.

    if stake > max_stake {
        return QUIC_MAX_STAKED_RECEIVE_WINDOW_RATIO;
    }

    let max_ratio = QUIC_MAX_STAKED_RECEIVE_WINDOW_RATIO;
    let min_ratio = QUIC_MIN_STAKED_RECEIVE_WINDOW_RATIO;
    if max_stake > min_stake {
        let a = (max_ratio - min_ratio) as f64 / (max_stake - min_stake) as f64;
        let b = max_ratio as f64 - ((max_stake as f64) * a);
        let ratio = (a * stake as f64) + b;
        ratio.round() as u64
    } else {
        QUIC_MAX_STAKED_RECEIVE_WINDOW_RATIO
    }
}
