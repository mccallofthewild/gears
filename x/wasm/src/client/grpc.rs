//! gRPC service definitions for the wasm module.
//!
//! This mirrors the server implementations found in
//! [`wasmd`](https://github.com/CosmWasm/wasmd/tree/main/x/wasm)
//! and exposes query helpers to external tooling. The service is generic over a
//! [`NodeQueryHandler`] so it can be wired into any `BaseApp` instance.

use gears::baseapp::{NodeQueryHandler, QueryRequest, QueryResponse};
use ibc_proto::cosmwasm::wasm::v1::{
    query_server::{Query, QueryServer},
    QueryCodeRequest as RawQueryCodeRequest, QueryCodeResponse as RawQueryCodeResponse,
    QueryCodesRequest as RawQueryCodesRequest, QueryCodesResponse as RawQueryCodesResponse,
    QueryContractInfoRequest as RawQueryContractInfoRequest,
    QueryContractInfoResponse as RawQueryContractInfoResponse,
    QueryContractsByCodeRequest as RawQueryContractsByCodeRequest,
    QueryContractsByCodeResponse as RawQueryContractsByCodeResponse,
    QueryRawContractStateRequest as RawQueryRawContractStateRequest,
    QueryRawContractStateResponse as RawQueryRawContractStateResponse,
    QuerySmartContractStateRequest as RawQuerySmartContractStateRequest,
    QuerySmartContractStateResponse as RawQuerySmartContractStateResponse,
};
use std::marker::PhantomData;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{WasmNodeQueryRequest, WasmNodeQueryResponse, WasmQuery};
use gears::baseapp::LatestHeight;

const ERROR_STATE_MSG: &str = "An internal error occurred while querying the application state.";

#[derive(Debug, Default)]
pub struct WasmService<QH, QReq, QRes> {
    app: QH,
    _phantom: PhantomData<(QReq, QRes)>,
}

#[tonic::async_trait]
impl<QReq, QRes, QH> Query for WasmService<QH, QReq, QRes>
where
    QReq: QueryRequest + From<WasmNodeQueryRequest> + Send + Sync + 'static,
    QRes: QueryResponse + TryInto<WasmNodeQueryResponse, Error = Status> + Send + Sync + 'static,
    QH: NodeQueryHandler<QReq, QRes> + LatestHeight + Send + Sync + 'static,
{
    async fn contract_info(
        &self,
        request: Request<RawQueryContractInfoRequest>,
    ) -> Result<Response<RawQueryContractInfoResponse>, Status> {
        info!("Received gRPC request wasm::contract_info");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::ContractInfo(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let WasmNodeQueryResponse::ContractInfo(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn code(
        &self,
        request: Request<RawQueryCodeRequest>,
    ) -> Result<Response<RawQueryCodeResponse>, Status> {
        info!("Received gRPC request wasm::code");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::Code(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;
        if let WasmNodeQueryResponse::Code(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn codes(
        &self,
        request: Request<RawQueryCodesRequest>,
    ) -> Result<Response<RawQueryCodesResponse>, Status> {
        info!("Received gRPC request wasm::codes");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::Codes(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let WasmNodeQueryResponse::Codes(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn contracts_by_code(
        &self,
        request: Request<RawQueryContractsByCodeRequest>,
    ) -> Result<Response<RawQueryContractsByCodeResponse>, Status> {
        info!("Received gRPC request wasm::contracts_by_code");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::ContractsByCode(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let WasmNodeQueryResponse::ContractsByCode(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn smart_contract_state(
        &self,
        request: Request<RawQuerySmartContractStateRequest>,
    ) -> Result<Response<RawQuerySmartContractStateResponse>, Status> {
        info!("Received gRPC request wasm::smart_contract_state");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::Smart(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let WasmNodeQueryResponse::Smart(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }

    async fn raw_contract_state(
        &self,
        request: Request<RawQueryRawContractStateRequest>,
    ) -> Result<Response<RawQueryRawContractStateResponse>, Status> {
        info!("Received gRPC request wasm::raw_contract_state");
        let req = WasmNodeQueryRequest {
            height: self.app.latest_height(),
            query: WasmQuery::Raw(request.into_inner().try_into()?),
        };
        let response: WasmNodeQueryResponse = self.app.typed_query(req)?.try_into()?;

        if let WasmNodeQueryResponse::Raw(resp) = response {
            Ok(Response::new(resp.into()))
        } else {
            Err(Status::internal(ERROR_STATE_MSG))
        }
    }
}

pub fn new<QH, QReq, QRes>(app: QH) -> QueryServer<WasmService<QH, QReq, QRes>>
where
    QReq: QueryRequest + Send + Sync + 'static + From<WasmNodeQueryRequest>,
    QRes: QueryResponse + Send + Sync + 'static + TryInto<WasmNodeQueryResponse, Error = Status>,
    QH: NodeQueryHandler<QReq, QRes> + LatestHeight,
{
    let wasm_service = WasmService {
        app,
        _phantom: PhantomData,
    };
    QueryServer::new(wasm_service)
}
