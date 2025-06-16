#![allow(clippy::useless_conversion)]
use std::collections::HashMap;
use std::str::FromStr;

use crate::application::ApplicationInfo;
use crate::baseapp::NodeQueryHandler;
use crate::rest::error::HTTPError;
use crate::types::pagination::request::PaginationRequest;
use crate::types::pagination::response::PaginationResponse;
use crate::types::request::tx::BroadcastTxRequest;
use crate::types::response::any::AnyTx;
use crate::types::response::block::GetBlockByHeightResponse;
use crate::types::response::node_info::{GetNodeInfoResponse, VersionInfo};
use crate::types::response::tx::{
    BroadcastTxResponse, BroadcastTxResponseLight, TxResponse, TxResponseLight,
};
use crate::types::response::tx_event::GetTxsEventResponse;
use crate::types::response::validators::GetLatestValidatorSetResponse;
use crate::types::tx::{Tx, TxMessage};
use axum::extract::{Path, Query as AxumQuery, State};
use axum::Json;
use bytes::Bytes;
use core_types::Protobuf;
use extensions::pagination::{
    IteratorPaginate, IteratorPaginateByOffset, PaginationByOffset, PaginationKey,
};
use ibc_proto::cosmos::tx::v1beta1::BroadcastMode;
use serde::Deserialize;
use tendermint::informal::Hash;
use tendermint::rpc::client::{Client, HttpClient, HttpClientUrl};
use tendermint::rpc::query::Query;
use tendermint::rpc::response::block::Response as BlockResponse;
use tendermint::rpc::response::tx::search::Response;
use tendermint::rpc::response::tx::Response as TxResponseRaw;
use tendermint::rpc::url::Url;
use tendermint::rpc::Order;
use tendermint::types::proto::block::Height;

use super::{parse_pagination, tendermint_events_handler::StrEventsHandler, Pagination, RestState};

pub async fn health(State(tendermint_rpc_address): State<HttpClientUrl>) -> Result<(), HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    client.health().await.map_err(|e| {
        tracing::error!("Error connecting to Tendermint: {e}");
        HTTPError::bad_gateway()
    })
}

pub async fn node_info<QReq, QRes, App: NodeQueryHandler<QReq, QRes> + ApplicationInfo>(
    State(state): State<RestState<QReq, QRes, App>>,
) -> Result<Json<GetNodeInfoResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(state.tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client.status().await.map_err(|e| {
        tracing::error!("Error connecting to Tendermint: {e}");
        HTTPError::gateway_timeout()
    })?;

    let node_info = GetNodeInfoResponse {
        default_node_info: Some(res.node_info.into()),
        // TODO: extend ApplicationInfo trait and add member to form the version info
        application_version: Some(VersionInfo {
            name: App::APP_NAME.to_string(),
            app_name: App::APP_NAME.to_string(),
            version: App::APP_VERSION.to_string(),
            git_commit: "".to_string(),
            build_tags: "".to_string(),
            rust_version: "1".to_string(),
            build_deps: vec![],
            cosmos_sdk_version: "".to_string(),
        }),
    };
    Ok(Json(node_info))
}

pub async fn validatorsets_latest(
    AxumQuery(pagination): AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetLatestValidatorSetResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let (page, limit) = parse_pagination(&pagination);
    let res = client
        .validators_latest(tendermint::rpc::client::Paging::Specific {
            page_number: (page as usize).into(),
            per_page: limit.into(),
        })
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })
        .map(|res| {
            let (pagination_result, iter) = res
                .validators
                .into_iter()
                .map(Into::into)
                .paginate_by_offset(PaginationByOffset::from((
                    page as usize - 1,
                    limit as usize,
                )));
            let validators = iter.collect();
            GetLatestValidatorSetResponse {
                block_height: res.block_height.into(),
                validators,
                pagination: Some(pagination_result.into()),
            }
        })?;
    Ok(Json(res))
}

pub async fn validatorsets(
    Path(height): Path<u32>,
    AxumQuery(pagination): AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetLatestValidatorSetResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let (page, limit) = parse_pagination(&pagination);
    let res = client
        .validators(
            height,
            tendermint::rpc::client::Paging::Specific {
                page_number: (page as usize).into(),
                per_page: limit.into(),
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })
        .map(|res| {
            let (pagination_result, iter) = res
                .validators
                .into_iter()
                .map(Into::into)
                .paginate_by_offset(PaginationByOffset::from((
                    page as usize - 1,
                    limit as usize,
                )));
            let validators = iter.collect();
            GetLatestValidatorSetResponse {
                block_height: res.block_height.into(),
                validators,
                pagination: Some(pagination_result.into()),
            }
        })?;

    Ok(Json(res))
}

#[derive(Deserialize)]
pub struct RawEvents {
    events: String,
}

pub async fn txs<M: TxMessage>(
    events: AxumQuery<RawEvents>,
    pagination: AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetTxsEventResponse<M>>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let queries = StrEventsHandler::new(&events.0.events)
        .try_parse_tendermint_events_vec()
        .map_err(|e| HTTPError::bad_request(e.to_string()))?;

    let query = Query::from_str(&queries.join(" AND "))
        .map_err(|e| HTTPError::bad_request(e.to_string()))?;
    let (page, limit) = parse_pagination(&pagination.0);

    let res_tx = client
        .tx_search(query, false, page, limit, Order::Descending)
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })?;

    let mut blocks: HashMap<Height, BlockResponse> = HashMap::with_capacity(res_tx.txs.len());
    for tx in &res_tx.txs {
        blocks.insert(
            tx.height,
            client.block(tx.height).await.map_err(|e| {
                tracing::error!("Error connecting to Tendermint: {e}");
                HTTPError::gateway_timeout()
            })?,
        );
    }

    let pagination = PaginationRequest::from(pagination.0);
    let res = map_responses(res_tx, blocks, pagination)?;

    Ok(Json(res))
}

pub async fn tx<M: TxMessage>(
    Path(hash): Path<Hash>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<BroadcastTxResponse<M>>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client.tx(hash, true).await.ok();
    let res = if let Some(r) = res {
        let time = client
            .block(r.height)
            .await
            .map(|block_res| block_res.block.header.time.to_string())
            .unwrap_or("unable to fetch transaction timestamp".to_string());
        Some(
            TxResponse::new_from_tx_response_and_string_time(r, time)
                .map_err(|_| HTTPError::internal_server_error())?,
        )
    } else {
        None
    };

    Ok(Json(BroadcastTxResponse {
        tx: res.as_ref().map(|r| r.tx.clone()).map(|tx| match tx {
            AnyTx::Tx(tx) => tx,
        }),
        tx_response: res,
    }))
}

pub async fn send_tx(
    state: State<HttpClientUrl>,
    tx_request: String,
) -> Result<Json<BroadcastTxResponseLight>, HTTPError> {
    let client = HttpClient::new::<Url>(state.0.clone().into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");
    let tx_request: BroadcastTxRequest =
        serde_json::from_str(&tx_request).map_err(|_| HTTPError::bad_gateway())?;

    let bytes = data_encoding::BASE64
        .decode(tx_request.tx_bytes.as_bytes())
        .map_err(|_| HTTPError::internal_server_error())?;

    let tx_response = if let Some(mode) = BroadcastMode::from_str_name(&tx_request.mode) {
        match mode {
            BroadcastMode::Sync => {
                let res = client
                    .broadcast_tx_sync(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.code.into(),
                    raw_log: res.log,
                }
            }
            BroadcastMode::Async => {
                let res = client
                    .broadcast_tx_async(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.code.into(),
                    raw_log: res.log,
                }
            }
            // TODO: is it a default value? keplr uses sync as default
            BroadcastMode::Block | BroadcastMode::Unspecified => {
                let res = client
                    .broadcast_tx_commit(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.deliver_tx.code.into(),
                    raw_log: res.deliver_tx.log,
                }
            }
        }
    } else {
        return Err(HTTPError::internal_server_error());
    };

    Ok(Json(BroadcastTxResponseLight {
        tx_response: Some(tx_response),
    }))
}

// wrapper allows to paginate response properly
// sorting of keys performs by height
#[derive(Clone)]
struct TxWrapper(TxResponseRaw);
impl PaginationKey for TxWrapper {
    fn iterator_key(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(i64::from(self.0.height).to_be_bytes().to_vec())
    }
}

// Maps a tendermint tx_search response to a Cosmos get txs by event response
fn map_responses<M: TxMessage>(
    res_tx: Response,
    blocks: HashMap<Height, BlockResponse>,
    pagination: PaginationRequest,
) -> Result<GetTxsEventResponse<M>, HTTPError> {
    let mut tx_responses = Vec::with_capacity(res_tx.txs.len());
    let mut txs = Vec::with_capacity(res_tx.txs.len());

    let (pagination_result, txs_iterator) = res_tx
        .txs
        .into_iter()
        .map(TxWrapper)
        .paginate(crate::extensions::pagination::Pagination::from(pagination));

    for tx in txs_iterator.map(|wrapped| wrapped.0) {
        let cosmos_tx = Tx::decode::<Bytes>(tx.tx.into()).map_err(|_| HTTPError::bad_gateway())?;
        txs.push(cosmos_tx.clone());

        let any_tx = AnyTx::Tx(cosmos_tx);
        let block_res = blocks
            .get(&tx.height)
            .expect("block with transaction height exists");

        tx_responses.push(TxResponse {
            height: tx.height.into(),
            txhash: tx.hash.to_string(),
            codespace: tx.tx_result.codespace,
            code: tx.tx_result.code.value(),
            data: hex::encode(tx.tx_result.data),
            raw_log: tx.tx_result.log.clone(),
            logs: tx.tx_result.log,
            info: tx.tx_result.info,
            gas_wanted: tx.tx_result.gas_wanted,
            gas_used: tx.tx_result.gas_used,
            tx: any_tx,
            timestamp: block_res.block.header.time.to_string(),
            events: tx.tx_result.events.into_iter().map(Into::into).collect(),
        });
    }

    let total = txs.len().try_into().map_err(|_| HTTPError::bad_gateway())?;

    Ok(GetTxsEventResponse {
        pagination: Some(PaginationResponse::from(pagination_result)),
        total,
        txs,
        tx_responses,
    })
}

pub async fn block(
    Path(height): Path<u32>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetBlockByHeightResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client
        .block(height)
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })
        .map(|res| GetBlockByHeightResponse {
            block_id: Some(res.block_id.into()),
            block: Some(res.block.clone()),
            sdk_block: Some(res.block),
        })?;
    Ok(Json(res))
}

pub async fn block_latest(
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetBlockByHeightResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client
        .latest_block()
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })
        .map(|res| GetBlockByHeightResponse {
            block_id: Some(res.block_id.into()),
            block: Some(res.block.clone()),
            sdk_block: Some(res.block),
        })?;
    Ok(Json(res))
}
