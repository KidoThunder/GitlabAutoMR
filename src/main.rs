use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要推送的分支名（MR模式必需）
    #[arg(short, long)]
    source_branch: Option<String>,

    /// 目标合并分支名（MR模式必需）
    #[arg(short, long)]
    target_branch: Option<String>,

    /// GitLab API URL (例如: https://gitlab.com/api/v4)
    #[arg(short = 'g', long)]
    gitlab_url: Option<String>,

    /// GitLab Group 路径（MR / Tag / approve-mrs 模式必需），例如: server/lobby
    #[arg(long)]
    group_path: Option<String>,

    /// GitLab API Token（可通过环境变量 GITLAB_TOKEN 设置）
    #[arg(short = 'k', long)]
    gitlab_token: Option<String>,

    /// 要打 tag 的分支名或 commit SHA（Tag模式必需）
    #[arg(short, long)]
    checkout_branch: Option<String>,

    /// 要创建的tag名（Tag模式必需）
    #[arg(long)]
    tag_name: Option<String>,

    /// tag的注释信息（Tag模式可选）
    #[arg(long)]
    tag_message: Option<String>,

    /// 操作模式：
    /// - mr: 创建 Merge Request
    /// - tag: 创建 Tag
    /// - list-mrs: 列出当前用户的 Merge Requests
    /// - approve-mrs: 批量“同意/批准”指定分支的 Merge Requests（调用 approve API）
    #[arg(short, long, default_value = "mr")]
    mode: String,

    /// MR状态筛选（list-mrs 模式使用）: opened, closed, locked, merged, all
    #[arg(long, default_value = "opened")]
    mr_state: String,
}

#[derive(Debug, Serialize)]
struct MergeRequestRequest {
    source_branch: String,
    target_branch: String,
    title: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateTagRequest {
    tag_name: String,
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitTag {
    name: String,
    target: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MergeRequest {
    web_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MergeRequestDetail {
    id: u64,
    iid: u64,
    title: String,
    state: String,
    source_branch: String,
    target_branch: String,
    web_url: String,
    created_at: String,
    updated_at: String,
    author: Author,
    project_id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Author {
    name: String,
    username: String,
}

struct GitLabClient {
    client: Client,
    base_url: String,
    token: String,
}

impl GitLabClient {
    fn new(base_url: String, token: String) -> Self {
        let client = Client::new();
        Self {
            client,
            base_url,
            token,
        }
    }

    async fn create_merge_request(
        &self,
        project_id: u64,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<MergeRequest> {
        let url = format!("{}/projects/{}/merge_requests", self.base_url, project_id);
        
        let request_body = MergeRequestRequest {
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            title: title.to_string(),
            description: Some(format!("Auto-created merge request from {} to {}", source_branch, target_branch)),
        };

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to create merge request: {}", error_text));
        }

        let merge_request: MergeRequest = response.json().await?;
        Ok(merge_request)
    }

    async fn create_tag(
        &self,
        project_id: u64,
        tag_name: &str,
        ref_name: &str,
        message: Option<&str>,
    ) -> Result<GitTag> {
        let url = format!(
            "{}/projects/{}/repository/tags",
            self.base_url, project_id
        );

        let request_body = CreateTagRequest {
            tag_name: tag_name.to_string(),
            ref_name: ref_name.to_string(),
            message: message.map(|m| m.to_string()),
        };

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Failed to create tag `{}` from ref `{}` (HTTP {}): {}",
                tag_name,
                ref_name,
                status.as_u16(),
                error_text
            ));
        }

        let tag: GitTag = response.json().await?;
        Ok(tag)
    }

    async fn list_my_merge_requests(&self, state: &str) -> Result<Vec<MergeRequestDetail>> {
        let url = format!("{}/merge_requests", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[("scope", "created_by_me"), ("state", state)])
            .send()
            .await?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Failed to list merge requests (HTTP {}): {}",
                status.as_u16(),
                error_text
            ));
        }

        let merge_requests: Vec<MergeRequestDetail> = response.json().await?;
        Ok(merge_requests)
    }

    async fn list_project_merge_requests(
        &self,
        project_id: u64,
        state: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<MergeRequestDetail>> {
        let url = format!("{}/projects/{}/merge_requests", self.base_url, project_id);

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("state", state),
                ("source_branch", source_branch),
                ("target_branch", target_branch),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Failed to list project merge requests (HTTP {}): {}",
                status.as_u16(),
                error_text
            ));
        }

        let merge_requests: Vec<MergeRequestDetail> = response.json().await?;
        Ok(merge_requests)
    }

    async fn list_group_projects(&self, group_path: &str) -> Result<Vec<Project>> {
        let url = format!(
            "{}/groups/{}/projects",
            self.base_url,
            urlencoding::encode(group_path)
        );

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[("include_subgroups", "true"), ("per_page", "100")])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Failed to list group projects for `{}` (HTTP {}): {}",
                group_path,
                status.as_u16(),
                error_text
            ));
        }

        let projects: Vec<Project> = response.json().await?;
        Ok(projects)
    }

    async fn merge_merge_request(&self, project_id: u64, mr_iid: u64) -> Result<()> {
        let url = format!(
            "{}/projects/{}/merge_requests/{}/approve",
            self.base_url, project_id, mr_iid
        );

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Failed to approve merge request #{} (HTTP {}): {}",
                mr_iid,
                status.as_u16(),
                error_text
            ));
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let token = if args.mode == "mr"
        || args.mode == "tag"
        || args.mode == "list-mrs"
        || args.mode == "approve-mrs"
    {
        args.gitlab_token
            .or_else(|| std::env::var("GITLAB_TOKEN").ok())
            .ok_or_else(|| {
                anyhow!("GitLab token required. Set GITLAB_TOKEN or use --gitlab-token")
            })?
    } else {
        String::new()
    };

    match args.mode.as_str() {
        "mr" => {
            let source_branch = args.source_branch.ok_or_else(|| {
                anyhow!("在MR模式下，必须指定 --source-branch 参数")
            })?;
            let target_branch = args.target_branch.ok_or_else(|| {
                anyhow!("在MR模式下，必须指定 --target-branch 参数")
            })?;
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在MR模式下，必须指定 --gitlab-url 参数")
            })?;
            let group_path = args.group_path.ok_or_else(|| {
                anyhow!("在 MR 模式下，必须指定 --group-path 参数，例如 server/lobby")
            })?;

            let gitlab_client = GitLabClient::new(gitlab_url, token);
            let projects = gitlab_client.list_group_projects(&group_path).await?;

            for project in projects {
                let _ = process_project_for_mr(&project, &source_branch, &target_branch, &gitlab_client).await;
            }
        }
        "tag" => {
            let checkout_branch = args.checkout_branch.ok_or_else(|| {
                anyhow!("在 tag 模式下，必须指定 --checkout-branch 参数")
            })?;
            let tag_name = args.tag_name.ok_or_else(|| {
                anyhow!("在tag模式下，必须指定 --tag-name 参数")
            })?;
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在 tag 模式下，必须指定 --gitlab-url 参数")
            })?;
            let group_path = args.group_path.ok_or_else(|| {
                anyhow!("在 tag 模式下，必须指定 --group-path 参数，例如 server/lobby")
            })?;

            let gitlab_client = GitLabClient::new(gitlab_url, token);
            let projects = gitlab_client.list_group_projects(&group_path).await?;

            for project in projects {
                let _ = process_project_for_tag(
                    &project,
                    &checkout_branch,
                    &tag_name,
                    args.tag_message.as_deref(),
                    &gitlab_client,
                )
                .await;
            }
        }
        "list-mrs" => {
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在list-mrs模式下，必须指定 --gitlab-url 参数")
            })?;

            let gitlab_client = GitLabClient::new(gitlab_url, token);
            let merge_requests = gitlab_client.list_my_merge_requests(&args.mr_state).await?;

            if merge_requests.is_empty() {
                return Ok(());
            }

            for mr in merge_requests {
                eprintln!(
                    "MR #{} | {} | {} → {} | {}",
                    mr.iid, mr.title, mr.source_branch, mr.target_branch, mr.web_url
                );
            }
        }
        "approve-mrs" => {
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在 approve-mrs 模式下，必须指定 --gitlab-url 参数")
            })?;
            let group_path = args.group_path.ok_or_else(|| {
                anyhow!("在 approve-mrs 模式下，必须指定 --group-path 参数，例如 server/lobby")
            })?;

            let source_branch = args.source_branch.unwrap_or_else(|| "dev".to_string());
            let target_branch = args.target_branch.unwrap_or_else(|| "release".to_string());

            let gitlab_client = GitLabClient::new(gitlab_url, token);
            let projects = gitlab_client.list_group_projects(&group_path).await?;

            for project in projects {
                let _ = process_project_for_approve(
                    &project,
                    &source_branch,
                    &target_branch,
                    &gitlab_client,
                )
                .await;
            }
        }
        _ => {
            return Err(anyhow!(
                "不支持的模式: {}。支持的模式: mr, tag, list-mrs, approve-mrs",
                args.mode
            ));
        }
    }

    Ok(())
}

async fn process_project_for_mr(
    project: &Project,
    source_branch: &str,
    target_branch: &str,
    gitlab_client: &GitLabClient,
) -> Result<String> {
    let title = format!("{} to {}", source_branch, target_branch);
    let merge_request = gitlab_client
        .create_merge_request(project.id, source_branch, target_branch, &title)
        .await?;

    Ok(format!("{}: {}", project.name, merge_request.web_url))
}

async fn process_project_for_tag(
    project: &Project,
    ref_name: &str,
    tag_name: &str,
    tag_message: Option<&str>,
    gitlab_client: &GitLabClient,
) -> Result<String> {
    let tag = gitlab_client
        .create_tag(project.id, tag_name, ref_name, tag_message)
        .await?;

    Ok(format!(
        "{}: 成功在 {} 上创建 tag {} (目标: {})",
        project.name, ref_name, tag.name, tag.target
    ))
}

async fn process_project_for_approve(
    project: &Project,
    source_branch: &str,
    target_branch: &str,
    gitlab_client: &GitLabClient,
) -> Result<String> {
    let mrs = gitlab_client
        .list_project_merge_requests(project.id, "opened", source_branch, target_branch)
        .await?;

    if mrs.is_empty() {
        return Ok(format!(
            "{}: 无待审批的 MR ({} -> {})",
            project.name, source_branch, target_branch
        ));
    }

    let total = mrs.len();
    let mut approved = 0usize;

    for mr in mrs {
        if gitlab_client.merge_merge_request(project.id, mr.iid).await.is_ok() {
            approved += 1;
        }
    }

    Ok(format!(
        "{}: 成功批准 {}/{} 个 MR ({} -> {})",
        project.name, approved, total, source_branch, target_branch
    ))
}
