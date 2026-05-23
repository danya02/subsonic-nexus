use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getUser
pub async fn get_user(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getUsers
pub async fn get_users(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createUser
pub async fn create_user(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/updateUser
pub async fn update_user(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deleteUser
pub async fn delete_user(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/changePassword
pub async fn change_password(_auth: SubsonicAuth) -> Response {
    todo!()
}
