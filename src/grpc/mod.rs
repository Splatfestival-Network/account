use tonic::{async_trait, Request, Response, Status};
use crate::grpc::grpc::{ExchangeTokenForUserDataRequest, GetNexDataRequest, GetNexDataResponse, GetNexPasswordRequest, GetNexPasswordResponse, GetUserDataRequest, GetUserDataResponse, UpdatePnidPermissionsRequest};
use crate::Pool;

mod grpc {
    tonic::include_proto!("account");
}


pub struct AccountService(pub Pool);

#[async_trait]
impl grpc::account_server::Account for AccountService{
    async fn exchange_token_for_user_data(&self, request: Request<ExchangeTokenForUserDataRequest>) -> Result<Response<GetUserDataResponse>, Status> {
        todo!()
    }
    async fn get_nex_data(&self, request: Request<GetNexDataRequest>) -> Result<Response<GetNexDataResponse>, Status> {
        todo!()
    }
    async fn get_nex_password(&self, request: Request<GetNexPasswordRequest>) -> Result<Response<GetNexPasswordResponse>, Status> {
        todo!()
    }
    async fn update_pnid_permissions(&self, request: Request<UpdatePnidPermissionsRequest>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_user_data(&self, request: Request<GetUserDataRequest>) -> Result<Response<GetUserDataResponse>, Status> {
        todo!()
    }
}