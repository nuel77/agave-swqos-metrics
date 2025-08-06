use crate::utils::{calculate_stake_stats, compute_recieve_window};
use clap::Parser;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::packet::PACKET_DATA_SIZE;
use solana_sdk::pubkey::Pubkey;
use solana_streamer::nonblocking::quic::{ConnectionPeerType, compute_max_allowed_uni_streams};
use std::collections::HashMap;
use std::ops::Div;
use std::str::FromStr;
use std::sync::Arc;

pub mod utils;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    rpc_url: String,
    #[arg(long)]
    validator_key: String,
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let rpc = RpcClient::new(args.rpc_url);

    let vote_accounts = rpc.get_vote_accounts().await.unwrap();
    let stake_map = Arc::new(
        vote_accounts
            .current
            .iter()
            .chain(vote_accounts.delinquent.iter())
            .filter_map(|vote_account| {
                Some((
                    Pubkey::from_str(&vote_account.node_pubkey).ok()?,
                    vote_account.activated_stake,
                ))
            })
            .collect::<HashMap<Pubkey, u64>>(),
    );
    // stakes have values right now
    let (total_stake, min_stake, max_stake) = calculate_stake_stats(&stake_map, &HashMap::new());
    println!(
        "Network stats: total stake, min_stake, max_stake : {:?}",
        (total_stake, min_stake, max_stake)
    );

    let validator_key = Pubkey::from_str(&args.validator_key).unwrap();
    let our_stake = *stake_map.get(&validator_key).unwrap();
    let peer_type = ConnectionPeerType::Staked(our_stake);
    let max_uni_streams = compute_max_allowed_uni_streams(peer_type, total_stake);
    let receive_window: u64 = compute_recieve_window(max_stake, min_stake, peer_type)
        .unwrap()
        .into_inner();
    println!("max uni streams: {:?}", max_uni_streams);
    let n_txns = receive_window.div(PACKET_DATA_SIZE as u64);
    println!(
        "receive_window: {:?} bytes, {:?} max sized transactions",
        receive_window, n_txns
    );
}
