//! The graphql mutation
use anyhow::anyhow;
use async_graphql::{Context, Object, Result as GraphqlResult, Upload};
use tokio::fs::File;
use tokio::sync::mpsc::Sender;
use tokio_tar::Archive;

use crate::test::TestTask;

use super::model::FlatTestResults;

pub(in crate::webserver) struct TestMutation;

#[Object]
impl TestMutation {
    /// Test the provided probe-rs project
    async fn test(
        &self,
        ctx: &Context<'_>,
        tarball: Upload, /*... TODO add testoptions, like target/probe filters etc. */
    ) -> GraphqlResult<FlatTestResults> {
        let file = tarball.value(ctx)?;

        if file.content_type != Some("appplication/octet".to_owned())
            && file.filename.split('.').last() != Some("tar")
        {
            return Err(anyhow!("Invalid file format. Expecting tar archive.").into());
        }

        let test_task_sender = ctx.data::<Sender<TestTask>>().unwrap();

        let archive = Archive::new(File::from_std(file.content));

        let (test_task, test_result_receiver) = TestTask::new(archive);

        test_task_sender.send(test_task).await?;

        let results = test_result_receiver.await?;

        Ok(results.into())
    }
}
