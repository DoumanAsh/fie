use std::error::Error;
use std::fmt;

#[repr(u8)]
#[derive(Debug)]
///Gab errors
pub enum GabError {
    ///Authorization failed.
    LoginFailed,
    ///Failed to send request to upload image.
    ImageUploadSendError,
    ///Server rejected image upload.
    ImageUploadServerReject,
    ///Server responded with invalid data.
    ///
    ///Should contain `id`
    ImageUploadInvalidResponse,
    ///Failed to send request to perform text post.
    PostUploadSendError,
    ///Server rejected posting.
    PostUploadServerReject,
    ///Server responded with invalid data
    ///
    ///Should contain `id`
    PostUploadInvalidResponse,

}

impl fmt::Display for GabError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for GabError {
    fn description(&self) -> &str {
        match self {
            &GabError::LoginFailed => "Login has failed",
            &GabError::ImageUploadSendError => "Failed to send request to upload image",
            &GabError::ImageUploadServerReject => "Server rejected upload of image",
            &GabError::ImageUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
            &GabError::PostUploadSendError => "Failed to send request to perform text post",
            &GabError::PostUploadServerReject => "Server rejected posting",
            &GabError::PostUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
        }
    }
}
