use crate::Pool;
use crate::grpc::grpc::{
    ExchangeTokenForUserDataRequest, GetNexDataRequest, GetNexDataResponse, GetNexPasswordRequest,
    GetNexPasswordResponse, GetUserDataRequest, GetUserDataResponse, UpdatePnidPermissionsRequest,
};
use once_cell::sync::Lazy;
use std::env;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status, async_trait};

/// This module is a legacy module meant for interacting with existing pretendo
/// servers. This will inevitably be removed completely as this is only meant as
/// a stopgap until RNEX is in a fully functional state.

pub mod grpc {
    tonic::include_proto!("account");
}

static GRPC_PASSWORD: Lazy<Box<str>> = Lazy::new(|| {
    env::var("GRPC_PASSWORD")
        .expect("GRPC_PASSWORD not specified")
        .into_boxed_str()
});

fn verify_grpc_key(meta: &MetadataMap) -> Result<(), Status> {
    // req.metadata_mut().insert("x-api-key", API_KEY.clone());

    let key = meta
        .get("x-api-key")
        .ok_or(Status::permission_denied("api key missing"))?;

    if key.as_bytes() != GRPC_PASSWORD.as_bytes() {
        return Err(Status::permission_denied("GO AWAY"));
    }

    Ok(())
}

pub struct AccountService(pub Pool);

#[async_trait]
impl grpc::account_server::Account for AccountService {
    async fn exchange_token_for_user_data(
        &self,
        request: Request<ExchangeTokenForUserDataRequest>,
    ) -> Result<Response<GetUserDataResponse>, Status> {
        verify_grpc_key(request.metadata())?;

        Err(Status::unimplemented(
            "grpc tecnically isnt supported by account-rs as such no full support is guaranteed(you called a stubbed function)",
        ))
    }
    async fn get_nex_data(
        &self,
        request: Request<GetNexDataRequest>,
    ) -> Result<Response<GetNexDataResponse>, Status> {
        verify_grpc_key(request.metadata())?;

        Err(Status::unimplemented(
            "grpc tecnically isnt supported by account-rs as such no full support is guaranteed(you called a stubbed function)",
        ))
    }
    async fn get_nex_password(
        &self,
        request: Request<GetNexPasswordRequest>,
    ) -> Result<Response<GetNexPasswordResponse>, Status> {
        verify_grpc_key(request.metadata())?;

        let data = request.get_ref();

        let password = sqlx::query!(
            "select nex_password from users where pid = $1",
            data.pid as i32
        )
        .fetch_one(&self.0)
        .await
        .map_err(|_| Status::invalid_argument("No NEX account found"))?
        .nex_password;

        Ok(Response::new(GetNexPasswordResponse { password }))
    }
    async fn update_pnid_permissions(
        &self,
        request: Request<UpdatePnidPermissionsRequest>,
    ) -> Result<Response<()>, Status> {
        verify_grpc_key(request.metadata())?;

        Err(Status::unimplemented(
            "grpc tecnically isnt supported by account-rs as such no full support is guaranteed(you called a stubbed function)",
        ))
    }

    async fn get_user_data(
        &self,
        request: Request<GetUserDataRequest>,
    ) -> Result<Response<GetUserDataResponse>, Status> {
        verify_grpc_key(request.metadata())?;

        Err(Status::unimplemented(
            "grpc tecnically isnt supported by account-rs as such no full support is guaranteed(you called a stubbed function)",
        ))
    }
}
