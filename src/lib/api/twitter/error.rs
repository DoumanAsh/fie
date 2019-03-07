use std::error::Error;
use std::fmt;

#[repr(u8)]
#[derive(Debug)]
///Twitter errors
pub enum TwitterError {
    ///Provided consumer and access tokens are invalid
    ///
    ///Possible reasons:
    ///
    ///- Empty values
    InvalidAuthData,
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

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for TwitterError {
    fn description(&self) -> &str {
        match self {
            &TwitterError::InvalidAuthData => "Provided consume and/or access tokens are invalid",
            &TwitterError::ImageUploadSendError => "Failed to send request to upload image",
            &TwitterError::ImageUploadServerReject => "Server rejected upload of image",
            &TwitterError::ImageUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
            &TwitterError::PostUploadSendError => "Failed to send request to perform text post",
            &TwitterError::PostUploadServerReject => "Server rejected posting",
            &TwitterError::PostUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
        }
    }
}
