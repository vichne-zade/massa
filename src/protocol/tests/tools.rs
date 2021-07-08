use super::super::{config::ProtocolConfig, protocol_controller::NodeId};
use super::mock_network_controller::{MockNetworkCommand, MockNetworkControllerInterface};
use crate::crypto::signature::{PrivateKey, SignatureEngine};
use rand::{rngs::StdRng, FromEntropy};
use std::time::Duration;

// generate random node ID (public key) and private key
pub fn generate_node_keys() -> (PrivateKey, NodeId) {
    let signature_engine = SignatureEngine::new();
    let mut rng = StdRng::from_entropy();
    let private_key = SignatureEngine::generate_random_private_key(&mut rng);
    let self_node_id = NodeId(signature_engine.derive_public_key(&private_key));
    (private_key, self_node_id)
}

// create a ProtocolConfig with typical values
pub fn create_protocol_config() -> ProtocolConfig {
    ProtocolConfig {
        message_timeout: Duration::from_secs(5),
        ask_peer_list_interval: Duration::from_secs(50),
    }
}

// ignore all commands while waiting for a futrue
pub async fn ignore_commands_while<FutureT: futures::Future + Unpin>(
    mut future: FutureT,
    mock_network_interface: &mut MockNetworkControllerInterface,
) -> FutureT::Output {
    loop {
        tokio::select!(
            res = &mut future => return res,
            cmd = mock_network_interface.wait_command() => match cmd {
                Some(MockNetworkCommand::GetAdvertisablePeerList(sender_tx)) => sender_tx.send(vec![]).unwrap(),
                Some(_) => {},
                None => return future.await,  // if the network controlled dies, wait for the future to finish
            }
        );
    }
}
