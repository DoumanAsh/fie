use std::error::Error;
use std::fmt;

#[repr(u8)]
#[derive(Debug)]
///Minds errors
pub enum MindsError {
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

impl fmt::Display for MindsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for MindsError {
    fn description(&self) -> &str {
        match self {
            &MindsError::LoginFailed => "Login has failed",
            &MindsError::ImageUploadSendError => "Failed to send request to upload image",
            &MindsError::ImageUploadServerReject => "Server rejected upload of image",
            &MindsError::ImageUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
            &MindsError::PostUploadSendError => "Failed to send request to perform text post",
            &MindsError::PostUploadServerReject => "Server rejected posting",
            &MindsError::PostUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
        }
    }
}
