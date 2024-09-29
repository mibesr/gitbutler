use anyhow::Result;
use gitbutler_branch::VirtualBranchesHandle;
use gitbutler_command_context::CommandContext;
use gitbutler_patch_reference::{CommitOrChangeId, PatchReference};
use gitbutler_repo::{LogUntil, RepositoryExt as _};
use gitbutler_stack::Stack;
use tempfile::TempDir;

#[test]
fn init_success() -> Result<()> {
    let (ctx, _temp_dir) = command_ctx("multiple-commits")?;
    let test_ctx = test_ctx(&ctx)?;
    let mut branch = test_ctx.branch;
    assert!(!branch.initialized());
    assert_eq!(branch.heads.len(), 0);
    let result = branch.init(&ctx);
    assert!(result.is_ok());
    assert!(branch.initialized());
    assert_eq!(branch.heads.len(), 1);
    assert_eq!(branch.heads[0].name, "virtual");
    assert_eq!(
        branch.heads[0].target,
        CommitOrChangeId::CommitId(branch.head.to_string())
    );
    assert_eq!(branch, test_ctx.handle.get_branch(branch.id)?);
    Ok(())
}

#[test]
fn init_already_initialized_fails() -> Result<()> {
    let (ctx, _temp_dir) = command_ctx("multiple-commits")?;
    let test_ctx = test_ctx(&ctx)?;
    let mut branch = test_ctx.branch;
    let result = branch.init(&ctx);
    assert!(result.is_ok());
    let result = branch.init(&ctx);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn add_branch_success() -> Result<()> {
    let (ctx, _temp_dir) = command_ctx("multiple-commits")?;
    let mut test_ctx = test_ctx(&ctx)?;
    test_ctx.branch.init(&ctx)?;
    let reference = PatchReference {
        name: "asdf".into(),
        target: CommitOrChangeId::CommitId(test_ctx.commits[0].id().to_string()),
    };
    let result = test_ctx.branch.add_branch(&ctx, reference);
    assert!(result.is_ok());
    assert_eq!(test_ctx.branch.heads.len(), 2);
    Ok(())
}

#[test]
fn add_branch_uninitialized_fails() -> Result<()> {
    let (ctx, _temp_dir) = command_ctx("multiple-commits")?;
    let mut test_ctx = test_ctx(&ctx)?;
    let reference = PatchReference {
        name: "asdf".into(),
        target: CommitOrChangeId::CommitId(test_ctx.commits[0].id().to_string()),
    };
    let result = test_ctx.branch.add_branch(&ctx, reference);
    assert_eq!(
        result.err().unwrap().to_string(),
        "Stack has not been initialized"
    );
    Ok(())
}

#[test]
fn add_branch_invalid_name_fails() -> Result<()> {
    let (ctx, _temp_dir) = command_ctx("multiple-commits")?;
    let mut test_ctx = test_ctx(&ctx)?;
    test_ctx.branch.init(&ctx)?;
    let reference = PatchReference {
        name: "name with spaces".into(),
        target: CommitOrChangeId::CommitId(test_ctx.commits[0].id().to_string()),
    };
    let result = test_ctx.branch.add_branch(&ctx, reference);
    assert_eq!(result.err().unwrap().to_string(), "Invalid branch name");
    Ok(())
}

fn command_ctx(name: &str) -> Result<(CommandContext, TempDir)> {
    gitbutler_testsupport::writable::fixture("stacking.sh", name)
}

fn test_ctx(ctx: &CommandContext) -> Result<TestContext> {
    let handle = VirtualBranchesHandle::new(ctx.project().gb_dir());
    let branches = handle.list_all_branches()?;
    let branch = branches.iter().find(|b| b.name == "virtual").unwrap();
    let other_branch = branches.iter().find(|b| b.name != "virtual").unwrap();
    let target = handle.get_default_target()?;
    let branch_commits = ctx
        .repository()
        .log(branch.head, LogUntil::Commit(target.sha))?;
    let other_commits = ctx
        .repository()
        .log(other_branch.head, LogUntil::Commit(target.sha))?;
    Ok(TestContext {
        branch: branch.clone(),
        commits: branch_commits,
        // other_branch: other_branch.clone(),
        other_commits,
        handle,
    })
}
struct TestContext<'a> {
    branch: gitbutler_branch::Branch,
    commits: Vec<git2::Commit<'a>>,
    #[allow(dead_code)]
    other_commits: Vec<git2::Commit<'a>>,
    handle: VirtualBranchesHandle,
}
