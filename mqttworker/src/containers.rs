use std::io::{Read, Write};
use std::io::{stdout, stderr};
use std::sync::Arc;
use bollard::config::ContainerCreateBody;
use bollard::Docker;
use bollard::query_parameters::ListImagesOptionsBuilder;
use futures_util::{StreamExt, TryStreamExt};
use paho_mqtt::message;
use tokio::spawn;
use termion::raw::IntoRawMode;
use tokio::io::{AsyncWriteExt};
use messages::mqtt::ConnectedClient;
use crate::configuration::Task;
use sha2::{Sha256, Digest};


pub async fn run_task(task_definition: Task, mqttc: Arc<ConnectedClient>, parent_id: Option<String>){

    // Handle defaulting a parent id to 0 if None
    // This is a very rough and ready default
    let topic = match parent_id {
        None => {
            format!("workers/jobs/{}/{}_{}", uuid::Uuid::new_v4().to_string(),"0".to_string(), uuid::Uuid::new_v4().to_string())
        },
        Some(id) => {
            format!("workers/jobs/{}/{}_{}", uuid::Uuid::new_v4().to_string(), id, uuid::Uuid::new_v4().to_string())
        }
    };
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let image = &*task_definition.image;
    let _create_image_result =docker
        .create_image(
            Some(
                bollard::query_parameters::CreateImageOptionsBuilder::default().clone()
                    .from_image(image)
                    .build(),
            ),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await;

    let container_config = ContainerCreateBody {
        image: Some(String::from(image)),
        tty: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
        ..Default::default()
    };

    let id = docker
        .create_container(
            None::<bollard::query_parameters::CreateContainerOptions>,
            container_config,
        )
        .await.expect("")
        .id;
    docker
        .start_container(
            &id,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await.expect("");

    #[cfg(not(windows))]
    {
        let bollard::container::AttachContainerResults {
            mut output,
            mut input,
        } = docker
            .attach_container(
                &id,
                Some(
                    bollard::query_parameters::AttachContainerOptionsBuilder::default()
                        .stdout(true)
                        .stderr(true)
                        .stdin(true)
                        .stream(true)
                        .build(),
                ),
            )
            .await.expect("");
        // pipe stdin into the docker attach stream input
        spawn(async move {
            match task_definition.cmds {
                None => {}
                Some(task_defs) => {
                    for cmd in task_defs{
                        input.write_all(cmd.as_ref()).await.ok();
                        input.write("\n".as_bytes()).await.ok();
                    }
                    let final_exit = "exit\n".as_bytes();
                    input.write_all(final_exit.as_ref()).await.ok();
                }
            }
            }

        );

        // set stdout in raw mode so we can do tty stuff
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().expect("");
        let _stderr = stderr();
        // pipe docker attach output into stdout
        let stdout_topic = format!("{}/stdout", topic);
        while let Some(Ok(output)) = output.next().await {
            stdout.write_all(output.clone().into_bytes().as_ref()).expect("");
            mqttc.client.publish(message::Message::new(stdout_topic.clone(), output.into_bytes(), 0)).await.ok();
            stdout.flush().expect("");
        }
    }

    _= docker
        .remove_container(
            &id,
            Some(
                bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                    .force(true)
                    .build(),
            ),
        )
        .await;

}

pub async fn test_container() {

    // Use a connection function described above
    let docker = Docker::connect_with_podman_defaults();

    async move {
        let version = docker.unwrap().version().await.unwrap();
        println!("{:#?}", version);
    }.await;
}

pub async fn test_list_images() {
    let docker = Docker::connect_with_podman_defaults();

    async move {
        let options = ListImagesOptionsBuilder::default()
            .all(true)
            .build();
        let images = &docker.unwrap().list_images(Some(options)).await.unwrap();

        for image in images {
            println!("-> {:#?}", image);
        }
    }.await;
}

const IMAGE: &str = "alpine:3";
pub async fn test_launch_with_volumes() {
        let docker = Docker::connect_with_socket_defaults().unwrap();

        let _ =docker
            .create_image(
                Some(
                    bollard::query_parameters::CreateImageOptionsBuilder::default()
                        .from_image(IMAGE)
                        .build(),
                ),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await;

        let alpine_config = ContainerCreateBody {
            image: Some(String::from(IMAGE)),
            tty: Some(true),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };

        let id = docker
            .create_container(
                None::<bollard::query_parameters::CreateContainerOptions>,
                alpine_config,
            )
            .await.expect("")
            .id;
        docker
            .start_container(
                &id,
                None::<bollard::query_parameters::StartContainerOptions>,
            )
            .await.expect("");

        #[cfg(not(windows))]
        {
            let bollard::container::AttachContainerResults {
                mut output,
                mut input,
            } = docker
                .attach_container(
                    &id,
                    Some(
                        bollard::query_parameters::AttachContainerOptionsBuilder::default()
                            .stdout(true)
                            .stderr(true)
                            .stdin(true)
                            .stream(true)
                            .build(),
                    ),
                )
                .await.expect("");
            let cmd = "uname -a\n";
            // pipe stdin into the docker attach stream input
            spawn(async move {
                input.write_all(cmd.as_ref()).await.ok();
                let final_exit = "exit\n".as_bytes();
                input.write_all(final_exit.as_ref()).await.ok();
                }
            );

            // set stdout in raw mode so we can do tty stuff
            let stdout = stdout();
            let mut stdout = stdout.lock().into_raw_mode().expect("");

            let stderr = stderr();
            let _stderr = stderr.lock().into_raw_mode().expect("");

            // pipe docker attach output into stdout
            while let Some(Ok(output)) = output.next().await {
                stdout.write_all(output.into_bytes().as_ref()).expect("");
                stdout.flush().expect("");
            }
        }

        _= docker
            .remove_container(
                &id,
                Some(
                    bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                        .force(true)
                        .build(),
                ),
            )
            .await;
}