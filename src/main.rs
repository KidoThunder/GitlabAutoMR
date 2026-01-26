use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要遍历的根路径
    #[arg(short, long)]
    path: String,

    /// 要推送的分支名
    #[arg(short, long)]
    source_branch: String,

    /// 目标合并分支名
    #[arg(short, long)]
    target_branch: String,

    /// GitLab API URL (例如: https://gitlab.com/api/v4)
    #[arg(short, long)]
    gitlab_url: String,

    /// GitLab API Token
    #[arg(short, long)]
    gitlab_token: Option<String>,

    /// 是否强制推送
    #[arg(short, long, default_value = "false")]
    force: bool,
}

#[derive(Debug, Serialize)]
struct MergeRequestRequest {
    source_branch: String,
    target_branch: String,
    title: String,
    description: Option<String>,
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

    async fn get_project_by_path(&self, path: &str) -> Result<Project> {
        let url = format!("{}/projects/{}", self.base_url, urlencoding::encode(path));
        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get project: {}", response.status()));
        }

        let project: Project = response.json().await?;
        Ok(project)
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
}

fn find_git_repositories(root_path: &str) -> Result<Vec<String>> {
    let mut repositories = Vec::new();
    
    for entry in WalkDir::new(root_path)
        .max_depth(3) // 限制搜索深度
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repositories.push(parent.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(repositories)
}

fn get_remote_url(repo_path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get remote URL for {}", repo_path));
    }

    let url = String::from_utf8(output.stdout)?;
    Ok(url.trim().to_string())
}


fn extract_project_path_from_url(url: &str) -> Result<String> {
    // 处理不同的Git URL格式
    let path = if url.starts_with("git@") {
        // SSH格式: git@gitlab.com:group/project.git
        url.split(':')
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid SSH URL format"))?
            .trim_end_matches(".git")
    } else if url.contains("gitlab.com") {
        // HTTPS格式: https://gitlab.com/group/project.git
        url.split("gitlab.com/")
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid HTTPS URL format"))?
            .trim_end_matches(".git")
    } else {
        return Err(anyhow!("Unsupported Git URL format"));
    };

    Ok(path.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 获取GitLab token
    let token = if let Some(token) = args.gitlab_token {
        token
    } else {
        std::env::var("GITLAB_TOKEN").map_err(|_| {
            anyhow!("GitLab token not provided. Please set GITLAB_TOKEN environment variable or use --gitlab-token")
        })?
    };

    println!("🚀 开始批量创建Merge Request...");
    println!("📁 搜索路径: {}", args.path);
    println!("🌿 源分支: {}", args.source_branch);
    println!("🎯 目标分支: {}", args.target_branch);

    // 创建GitLab客户端
    let gitlab_client = GitLabClient::new(args.gitlab_url, token);

    // 查找所有git仓库
    let repositories = find_git_repositories(&args.path)?;
    println!("📦 找到 {} 个Git仓库", repositories.len());

    let mut results = Vec::new();

    for repo_path in repositories {
        println!("\n🔍 处理仓库: {}", repo_path);
        
        match process_repository(
            &repo_path,
            &args.source_branch,
            &args.target_branch,
            &gitlab_client,
        ).await {
            Ok(result) => {
                println!("✅ 成功: {}", result);
                results.push(result);
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
            }
        }
    }

    println!("\n📊 处理完成!");
    println!("✅ 成功创建 {} 个Merge Request", results.len());
    
    if !results.is_empty() {
        println!("\n📋 创建的Merge Request:");
        for result in results {
            println!("  - {}", result);
        }
    }

    Ok(())
}

async fn process_repository(
    repo_path: &str,
    source_branch: &str,
    target_branch: &str,
    gitlab_client: &GitLabClient,
) -> Result<String> {
    // 获取远程URL
    let remote_url = get_remote_url(repo_path)?;
    
    // 提取项目路径
    let project_path = extract_project_path_from_url(&remote_url)?;
    
    // 获取项目信息
    let project = gitlab_client.get_project_by_path(&project_path).await?;
    
    // 创建merge request
    let title = format!("{} to {}", source_branch, target_branch);
    let merge_request = gitlab_client
        .create_merge_request(project.id, source_branch, target_branch, &title)
        .await?;
    
    Ok(format!("{}: {}", project.name, merge_request.web_url))
}
