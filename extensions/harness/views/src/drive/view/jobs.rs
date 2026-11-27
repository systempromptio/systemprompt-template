//! The Drive calls the screen starts on tokio: listings, the folder tree,
//! everything a document's pane shows, the writes, and reading a file for
//! import. Each answers plain data the view applies on the UI thread.

use std::path::{Path, PathBuf};

use crate::drive::items::{
    CommentThread, DocumentItem, FolderNode, RevisionItem, ShareState, comment_threads,
    document_items, folder_tree,
};
use systemprompt_drive::{
    DriveClient, DriveError, ImportFile, ListQuery, ReadView, UploadProgress,
};
use systemprompt_identifiers::{DocumentCommentId, DocumentFolderId, DocumentId};
use systemprompt_models::documents::client::{
    CommentRequest, CreateDocumentRequest, CreateFolderRequest, DriveComment, DriveCreated,
    DriveDetail, DriveFolder,
};
use systemprompt_traits::CancellationToken;
use tokio::sync::watch;

use crate::drive::model::PreviewText;

const IMPORT_LIMIT: u64 = 512 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct ListingPage {
    pub(super) items: Vec<DocumentItem>,
    pub(super) next_cursor: Option<String>,
}

#[derive(Debug)]
pub(super) struct DocumentLoad {
    pub(super) detail: Result<DriveDetail, DriveError>,
    pub(super) preview: Result<PreviewText, DriveError>,
    pub(super) comments: Result<Vec<CommentThread>, DriveError>,
    pub(super) revisions: Result<Vec<RevisionItem>, DriveError>,
    pub(super) access: Result<ShareState, DriveError>,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ImportReadError {
    #[error("the file {} could not be read", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the file {} is larger than an import may be", path.display())]
    TooLarge { path: PathBuf },
}

pub(super) async fn page(client: DriveClient, query: ListQuery) -> Result<ListingPage, DriveError> {
    let listing = client.list_documents(&query).await?;
    Ok(ListingPage {
        items: document_items(&listing.documents),
        next_cursor: listing.next_cursor,
    })
}

pub(super) async fn folders(client: DriveClient) -> Result<Vec<FolderNode>, DriveError> {
    let folders = client.folders(None).await?;
    Ok(folder_tree(&folders))
}

pub(super) async fn document(client: DriveClient, id: DocumentId) -> DocumentLoad {
    let preview = async {
        let outline = client.read(&id, ReadView::Text).await?;
        Ok(PreviewText::from_outline(&outline))
    };
    let comments = async { client.comments(&id).await.map(comment_threads) };
    let (detail, preview, comments, revisions, access) = tokio::join!(
        client.document(&id),
        preview,
        comments,
        client.revisions(&id),
        client.access(&id)
    );
    DocumentLoad {
        detail,
        preview,
        comments,
        revisions,
        access,
    }
}

pub(super) async fn create_folder(
    client: DriveClient,
    name: String,
    parent_id: Option<DocumentFolderId>,
) -> Result<DriveFolder, DriveError> {
    client
        .create_folder(&CreateFolderRequest { name, parent_id })
        .await
}

pub(super) async fn create_document(
    client: DriveClient,
    request: CreateDocumentRequest,
) -> Result<DriveCreated, DriveError> {
    client.create_document(&request, None).await
}

pub(super) async fn comment(
    client: DriveClient,
    id: DocumentId,
    request: CommentRequest,
) -> Result<DriveComment, DriveError> {
    client.create_comment(&id, &request, None).await
}

pub(super) async fn resolve(
    client: DriveClient,
    id: DocumentId,
    comment: DocumentCommentId,
) -> Result<DriveComment, DriveError> {
    client.resolve_comment(&id, &comment).await
}

pub(super) async fn upload(
    client: DriveClient,
    file: ImportFile,
    progress: watch::Sender<Option<UploadProgress>>,
    cancel: CancellationToken,
) -> Result<DriveCreated, DriveError> {
    let report = move |step: UploadProgress| progress.send_modify(|slot| *slot = Some(step));
    client.import_file(file, &report, &cancel).await
}

pub(super) fn read_import(
    path: &Path,
    folder_id: Option<DocumentFolderId>,
) -> Result<ImportFile, ImportReadError> {
    let read_error = |source| ImportReadError::Read {
        path: path.to_path_buf(),
        source,
    };
    let length = std::fs::metadata(path).map_err(read_error)?.len();
    if length > IMPORT_LIMIT {
        return Err(ImportReadError::TooLarge {
            path: path.to_path_buf(),
        });
    }
    let bytes = std::fs::read(path).map_err(read_error)?;
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    Ok(ImportFile {
        mime: guess_mime(&name).map(str::to_owned),
        name,
        bytes: bytes.into(),
        folder_id,
        message: None,
        context_id: None,
    })
}

pub(super) fn guess_mime(name: &str) -> Option<&'static str> {
    let extension = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    let mime = match extension.as_str() {
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "csv" => "text/csv",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => return None,
    };
    Some(mime)
}
