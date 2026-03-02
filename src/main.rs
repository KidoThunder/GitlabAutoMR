use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要遍历的根路径（MR / Tag 模式必需，list-mrs / approve-mrs 模式不需要）
    #[arg(short, long)]
    path: Option<String>,

    /// 要推送的分支名（MR模式必需）
    #[arg(short, long)]
    source_branch: Option<String>,

    /// 目标合并分支名（MR模式必需）
    #[arg(short, long)]
    target_branch: Option<String>,

    /// GitLab API URL (例如: https://gitlab.com/api/v4)（MR模式必需）
    #[arg(short = 'g', long)]
    gitlab_url: Option<String>,

    /// GitLab Group 路径（approve-mrs 模式使用），例如: server/lobby
    #[arg(long)]
    group_path: Option<String>,

    /// GitLab API Token（MR模式可选）
    #[arg(short = 'k', long)]
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
struct MergeRequestDetail {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    project_id: u64,
}

#[derive(Debug, Deserialize)]
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

    async fn get_project_by_path(&self, path: &str) -> Result<Project> {
        let url = format!("{}/projects/{}", self.base_url, urlencoding::encode(path));
        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Failed to get project `{}` (HTTP {}): {}",
                path,
                status.as_u16(),
                body
            ));
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
                "Failed to list merge requests (HTTP {}): {}\n提示: 请确保 GitLab URL 正确，格式应为: https://your-gitlab.com/api/v4",
                status.as_u16(),
                error_text
            ));
        }

        // 先获取响应文本，用于调试
        let response_text = response.text().await?;
        
        // 尝试解析 JSON
        match serde_json::from_str::<Vec<MergeRequestDetail>>(&response_text) {
            Ok(merge_requests) => Ok(merge_requests),
            Err(e) => {
                // 显示前 200 个字符的响应内容用于调试
                let preview = if response_text.len() > 200 {
                    format!("{}...", &response_text[..200])
                } else {
                    response_text.clone()
                };
                Err(anyhow!(
                    "Failed to parse merge requests response: {}\nResponse preview: {}\n提示: 请确保 GitLab URL 正确，格式应为: https://your-gitlab.com/api/v4",
                    e,
                    preview
                ))
            }
        }
    }

    /// 列出指定项目中符合条件的 Merge Request
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

    /// 列出指定 Group 下的所有项目（包含子 Group）
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

    /// 批准（approve）指定的 Merge Request
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

    // 获取GitLab token（MR / list-mrs / approve-mrs 模式需要）
    let token = if args.mode == "mr" || args.mode == "list-mrs" || args.mode == "approve-mrs" {
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
    if let Some(ref path) = args.path {
        println!("📁 搜索路径: {}", path);
    }
    println!("🔧 操作模式: {}", args.mode);

    match args.mode.as_str() {
        "mr" => {
            let path = args.path.ok_or_else(|| {
                anyhow!("在MR模式下，必须指定 --path 参数")
            })?;
            
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
            let repositories = find_git_repositories(&path)?;
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
            let path = args.path.ok_or_else(|| {
                anyhow!("在tag模式下，必须指定 --path 参数")
            })?;
            
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
            let repositories = find_git_repositories(&path)?;
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
        "list-mrs" => {
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在list-mrs模式下，必须指定 --gitlab-url 参数")
            })?;
            
            println!("📋 MR状态筛选: {}", args.mr_state);
            
            // 创建GitLab客户端
            let gitlab_client = GitLabClient::new(gitlab_url, token);
            
            // 获取MR列表
            match gitlab_client.list_my_merge_requests(&args.mr_state).await {
                Ok(merge_requests) => {
                    println!("\n📊 找到 {} 个Merge Request", merge_requests.len());
                    
                    if merge_requests.is_empty() {
                        println!("暂无符合条件的Merge Request");
                    } else {
                        println!("\n📋 Merge Request列表:");
                        println!("{}", "=".repeat(100));
                        
                        for mr in merge_requests {
                            println!("\n🔹 MR #{} - {}", mr.iid, mr.title);
                            println!("   状态: {}", mr.state);
                            println!("   作者: {} (@{})", mr.author.name, mr.author.username);
                            println!("   分支: {} → {}", mr.source_branch, mr.target_branch);
                            println!("   创建时间: {}", mr.created_at);
                            println!("   更新时间: {}", mr.updated_at);
                            println!("   链接: {}", mr.web_url);
                        }
                        
                        println!("\n{}", "=".repeat(100));
                    }
                }
                Err(e) => {
                    println!("❌ 获取Merge Request列表失败: {}", e);
                    return Err(e);
                }
            }
        }
        "approve-mrs" => {
            let gitlab_url = args.gitlab_url.ok_or_else(|| {
                anyhow!("在 approve-mrs 模式下，必须指定 --gitlab-url 参数")
            })?;

            let group_path = args.group_path.ok_or_else(|| {
                anyhow!("在 approve-mrs 模式下，必须指定 --group-path 参数，例如 server/lobby")
            })?;

            // 默认为从 dev 到 release，可以通过参数覆盖
            let source_branch = args.source_branch.unwrap_or_else(|| "dev".to_string());
            let target_branch = args
                .target_branch
                .unwrap_or_else(|| "release".to_string());

            println!("📂 GitLab Group: {}", group_path);
            println!("🌿 源分支(待审批): {}", source_branch);
            println!("🎯 目标分支: {}", target_branch);

            // 创建GitLab客户端
            let gitlab_client = GitLabClient::new(gitlab_url, token);

            // 获取 Group 下的项目列表
            let projects = gitlab_client.list_group_projects(&group_path).await?;
            println!(
                "📦 在 group `{}` 下找到 {} 个项目",
                group_path,
                projects.len()
            );

            let mut results = Vec::new();

            for project in projects {
                println!("\n🔍 处理项目: {} (id: {})", project.name, project.id);

                match process_project_for_approve(
                    &project,
                    &source_branch,
                    &target_branch,
                    &gitlab_client,
                )
                .await
                {
                    Ok(result) => {
                        println!("✅ {}", result);
                        results.push(result);
                    }
                    Err(e) => {
                        println!("❌ 失败: {}", e);
                    }
                }
            }

            println!("\n📊 处理完成!");
            println!("✅ 已处理 {} 个项目的 Merge Request 审批", results.len());
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
    let project = match gitlab_client.get_project_by_path(&project_path).await {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            // 对于无权限/未授权的项目，记录并跳过，而不是中断整个批量操作
            if msg.contains("401") || msg.contains("Unauthorized") {
                println!(
                    "⚠️  无权限访问项目 `{}` (repo: {})，跳过该仓库。详情: {}",
                    project_path, repo_path, msg
                );
                return Ok(format!(
                    "{}: 无权限访问项目 `{}`，已跳过",
                    repo_path, project_path
                ));
            }
            // 其他错误继续向上抛出
            return Err(e);
        }
    };
    
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

/// 处理单个项目：查找并批准从 source_branch 到 target_branch 的 Merge Requests
async fn process_project_for_approve(
    project: &Project,
    source_branch: &str,
    target_branch: &str,
    gitlab_client: &GitLabClient,
) -> Result<String> {
    // 获取该项目中符合条件的 MR（默认仅处理打开状态的）
    let mrs = gitlab_client
        .list_project_merge_requests(project.id, "opened", source_branch, target_branch)
        .await?;

    if mrs.is_empty() {
        return Ok(format!(
            "{}: 无待审批的 MR ({} -> {})",
            project.name, source_branch, target_branch
        ));
    }

    println!(
        "📋 在项目 {} 中找到 {} 个待审批的 MR ({} -> {})",
        project.name,
        mrs.len(),
        source_branch,
        target_branch
    );

    let total = mrs.len();
    let mut approved = 0usize;

    for mr in mrs {
        println!(
            "  🔹 MR #{} - {} ({} -> {})",
            mr.iid, mr.title, mr.source_branch, mr.target_branch
        );
        match gitlab_client.merge_merge_request(project.id, mr.iid).await {
            Ok(()) => {
                println!("     ✅ 已批准 MR #{}: {}", mr.iid, mr.web_url);
                approved += 1;
            }
            Err(e) => {
                println!("     ❌ 合并 MR #{} 失败: {}", mr.iid, e);
            }
        }
    }

    Ok(format!(
        "{}: 成功批准 {}/{} 个 MR ({} -> {})",
        project.name, approved, total, source_branch, target_branch
    ))
}
