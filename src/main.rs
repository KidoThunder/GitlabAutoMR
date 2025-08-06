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

    /// 要推送的分支名（MR模式必需）
    #[arg(short, long)]
    source_branch: Option<String>,

    /// 目标合并分支名（MR模式必需）
    #[arg(short, long)]
    target_branch: Option<String>,

    /// GitLab API URL (例如: https://gitlab.com/api/v4)（MR模式必需）
    #[arg(short, long)]
    gitlab_url: Option<String>,

    /// GitLab API Token（MR模式可选）
    #[arg(short, long)]
    gitlab_token: Option<String>,

    /// 是否强制推送（MR模式可选）
    #[arg(short, long, default_value = "false")]
    force: bool,

    /// 要切换到的分支名（Tag模式必需）
    #[arg(short, long)]
    checkout_branch: Option<String>,

    /// 要创建的tag名（Tag模式必需）
    #[arg(long)]
    tag_name: Option<String>,

    /// tag的注释信息（Tag模式可选）
    #[arg(long)]
    tag_message: Option<String>,

    /// 操作模式：mr（创建merge request）或 tag（创建tag）
    #[arg(short, long, default_value = "mr")]
    mode: String,
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

fn checkout_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    // 切换到指定分支
    let output = Command::new("git")
        .args(["checkout", branch_name])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to checkout branch {} in {}", branch_name, repo_path));
    }

    println!("✅ 成功切换到分支: {}", branch_name);
    
    // 拉取最新代码
    let pull_output = Command::new("git")
        .args(["pull", "origin", branch_name])
        .current_dir(repo_path)
        .output()?;

    if !pull_output.status.success() {
        // 如果拉取失败，记录警告但不中断流程
        println!("⚠️  警告: 拉取分支 {} 最新代码失败，继续执行", branch_name);
        if let Ok(error_msg) = String::from_utf8(pull_output.stderr) {
            if !error_msg.trim().is_empty() {
                println!("   错误信息: {}", error_msg.trim());
            }
        }
    } else {
        println!("✅ 成功拉取分支 {} 的最新代码", branch_name);
    }
    
    Ok(())
}

fn create_tag(repo_path: &str, tag_name: &str, message: Option<&str>) -> Result<()> {
    let mut args = vec!["tag"];
    
    if let Some(msg) = message {
        args.extend_from_slice(&["-a", tag_name, "-m", msg]);
    } else {
        args.push(tag_name);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to create tag {} in {}", tag_name, repo_path));
    }

    println!("✅ 成功创建tag: {}", tag_name);
    Ok(())
}

fn push_tag(repo_path: &str, tag_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "origin", tag_name])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to push tag {} in {}", tag_name, repo_path));
    }

    println!("✅ 成功推送tag: {}", tag_name);
    Ok(())
}

fn get_current_branch(repo_path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get current branch in {}", repo_path));
    }

    let branch = String::from_utf8(output.stdout)?;
    Ok(branch.trim().to_string())
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

    // 获取GitLab token（仅MR模式需要）
    let token = if args.mode == "mr" {
        if let Some(token) = args.gitlab_token {
            token
        } else {
            std::env::var("GITLAB_TOKEN").map_err(|_| {
                anyhow!("GitLab token not provided. Please set GITLAB_TOKEN environment variable or use --gitlab-token")
            })?
        }
    } else {
        String::new() // Tag模式不需要token
    };

    println!("🚀 开始批量操作...");
    println!("📁 搜索路径: {}", args.path);
    println!("🔧 操作模式: {}", args.mode);

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
            
            println!("🌿 源分支: {}", source_branch);
            println!("🎯 目标分支: {}", target_branch);
            
            // 创建GitLab客户端
            let gitlab_client = GitLabClient::new(gitlab_url, token);

            // 查找所有git仓库
            let repositories = find_git_repositories(&args.path)?;
            println!("📦 找到 {} 个Git仓库", repositories.len());

            let mut results = Vec::new();

            for repo_path in repositories {
                println!("\n🔍 处理仓库: {}", repo_path);
                
                match process_repository(
                    &repo_path,
                    &source_branch,
                    &target_branch,
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
        }
        "tag" => {
            let checkout_branch = args.checkout_branch.ok_or_else(|| {
                anyhow!("在tag模式下，必须指定 --checkout-branch 参数")
            })?;
            
            let tag_name = args.tag_name.ok_or_else(|| {
                anyhow!("在tag模式下，必须指定 --tag-name 参数")
            })?;
            
            println!("🌿 切换分支: {}", checkout_branch);
            println!("🏷️ 创建tag: {}", tag_name);
            if let Some(ref msg) = args.tag_message {
                println!("📝 Tag消息: {}", msg);
            }

            // 查找所有git仓库
            let repositories = find_git_repositories(&args.path)?;
            println!("📦 找到 {} 个Git仓库", repositories.len());

            let mut results = Vec::new();

            for repo_path in repositories {
                println!("\n🔍 处理仓库: {}", repo_path);
                
                match process_repository_for_tag(
                    &repo_path,
                    &checkout_branch,
                    &tag_name,
                    args.tag_message.as_deref(),
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
            println!("✅ 成功创建 {} 个Tag", results.len());
            
            if !results.is_empty() {
                println!("\n📋 创建的Tag:");
                for result in results {
                    println!("  - {}", result);
                }
            }
        }
        _ => {
            return Err(anyhow!("不支持的模式: {}。支持的模式: mr, tag", args.mode));
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

async fn process_repository_for_tag(
    repo_path: &str,
    target_branch: &str,
    tag_name: &str,
    tag_message: Option<&str>,
) -> Result<String> {
    // 获取当前分支
    let current_branch = get_current_branch(repo_path)?;
    println!("📍 当前分支: {}", current_branch);
    
    // 切换到指定分支
    checkout_branch(repo_path, target_branch)?;
    
    // 创建tag
    create_tag(repo_path, tag_name, tag_message)?;
    
    // 推送tag
    push_tag(repo_path, tag_name)?;
    
    // 切换回原来的分支
    checkout_branch(repo_path, &current_branch)?;
    
    Ok(format!("{}: 成功创建并推送tag {}", repo_path, tag_name))
}
