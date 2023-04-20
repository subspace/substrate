// Copyright Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Block relay protocol related definitions.

use crate::service::network::NetworkServiceHandle;
use futures::channel::oneshot;
use libp2p::PeerId;
use sc_network::request_responses::{ProtocolConfig, RequestFailure};
use sp_runtime::traits::{Block as BlockT};
use std::sync::Arc;

/// The serving side of the block relay protocol. It runs a single instance
/// of the server task  that processes the incoming protocol messages.
#[async_trait::async_trait]
pub trait BlockServer<Block: BlockT>: Send {
	/// Starts the protocol processing.
	async fn run(&mut self);
}

/// The client side stub to download blocks from peers. This is a handle
/// that can be used to initiate concurrent downloads.
#[async_trait::async_trait]
pub trait BlockDownloader: Send + Sync {
	/// Performs the protocol specific sequence to fetch the block from the peer.
	/// Input: `request` is the serialized schema::v1::BlockRequest.
	/// Output: if the download succeeds, the serialized schema::v1::BlockResponse
	/// is returned.
	async fn download_block(
		&self,
		who: PeerId,
		request: Vec<u8>,
		network: NetworkServiceHandle,
	) -> Result<Result<Vec<u8>, RequestFailure>, oneshot::Canceled>;
}

/// Block relay specific params for network creation.
pub struct BlockRelayParams<Block: BlockT> {
	pub server: Box<dyn BlockServer<Block>>,
	pub downloader: Arc<dyn BlockDownloader>,
	pub request_response_config: ProtocolConfig,
}


