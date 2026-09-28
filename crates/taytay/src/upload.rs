use crate::{
    TaytayError,
    model::{ArtifactState, UploadJob},
    protocol::{LunsaranClient, TusClient},
    spool::Spool,
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub fn upload<C: LunsaranClient, T: TusClient>(
    spool: &Spool,
    client: &C,
    tus: &T,
    mut job: UploadJob,
    organization_id: &str,
    project_id: &str,
) -> Result<UploadJob, TaytayError> {
    spool.verify(&job)?;
    job.transition(ArtifactState::Uploading)?;
    job.attempts += 1;
    let session = client.create_upload_session(
        &job.artifact,
        organization_id,
        project_id,
        &job.artifact.id.0,
    )?;
    let upload_url = tus.create(&session, &job.artifact)?;
    let mut offset = tus.head(&upload_url)?.offset;
    let mut file = File::open(&job.artifact.path)?;
    file.seek(SeekFrom::Start(offset))?;
    let chunk_size = session.chunk_size.max(1) as usize;
    let mut buf = vec![0; chunk_size];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let result = tus.patch(
            &upload_url,
            offset,
            &buf[..n],
            job.artifact.checksum_sha256.as_deref(),
        )?;
        offset = result.offset;
    }
    job.server_url = Some(upload_url);
    job.offset = offset;
    job.transition(ArtifactState::Completed)?;
    client.report_state(&job.artifact, "completed")?;
    spool.ledger().update(job.clone())?;
    Ok(job)
}
