use std::error::Error;
use std::fmt;

#[repr(u8)]
#[derive(Debug)]
///Mastodon errors
pub enum MastodonError {
    ///Provided HOST URI is not valid URI.
    InvalidHostUri,
    ///Access token is invalid.
    ///
    ///Possible reasons:
    ///
    ///- Empty token
    InvalidToken,
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

impl fmt::Display for MastodonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for MastodonError {
    fn description(&self) -> &str {
        match self {
            &MastodonError::InvalidHostUri => "Provided Host URI is not valid URI",
            &MastodonError::InvalidToken => "Token is not valid(empty)",
            &MastodonError::ImageUploadSendError => "Failed to send request to upload image",
            &MastodonError::ImageUploadServerReject => "Server rejected upload of image",
            &MastodonError::ImageUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
            &MastodonError::PostUploadSendError => "Failed to send request to perform text post",
            &MastodonError::PostUploadServerReject => "Server rejected posting",
            &MastodonError::PostUploadInvalidResponse => "Server sent invalid response. Doesn't contain field id",
        }
    }
}
