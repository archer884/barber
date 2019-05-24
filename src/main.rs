mod fingerprint;
mod opt;

use fingerprint::LazyFingerprint;
use opt::Opt;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::{env, io};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let pwd = env::current_dir()?;

    // Create two lists of files to work from. The target tree is where these files all belong;
    // the context tree is where we want to search for potential duplicates. While the target
    // tree may be a subtree of the context, files within that subtree will not be checked.
    let target_tree = materialize_target_tree(opt.target(), &pwd);
    let context_tree = materialize_context_tree(opt.context().unwrap_or(&pwd), &target_tree, &pwd);

    // Build our lazy target set.
    let set = target_tree
        .iter()
        .filter_map(|path| LazyFingerprint::try_from_path(path).ok())
        .collect();

    // Print or remove duplicates in the broader context.
    if opt.force {
        remove_duplicates(&set, &context_tree, opt.silent, opt.debug)?;
    } else {
        list_duplicates(&set, &context_tree);
    }

    Ok(())
}

fn remove_duplicates(
    set: &HashSet<LazyFingerprint>,
    context: &[PathBuf],
    silent: bool,
    debug: bool,
) -> io::Result<()> {
    // Again, crazy nesting, I know. You just need to get over it.
    for item in context {
        if let Ok(candidate_fingerprint) = LazyFingerprint::try_from_path(&item) {
            if let Some(keep) = set.get(&candidate_fingerprint) {

                // Useful if you're paranoid, but I'd really like to just remove this feature...
                // Problem is, I've definitely seen one case where the list operation showed fewer
                // files than the removal operation removed.
                if !debug {
                    fs::remove_file(&item)?;
                } else {
                    println!("Kept {}", keep.path().display())
                }

                if !silent {
                    println!("Removed {}", item.display());
                }
            }
        }
    }
    Ok(())
}

fn list_duplicates(set: &HashSet<LazyFingerprint>, context: &[PathBuf]) {
    let mut duplicates_grouping: HashMap<_, Vec<_>> = HashMap::new();

    for item in context {
        // It looks weird to do this in two layers, but there's a reason behind it: some of these
        // paths refer to directories, not files, and those cannot (and should not) be converted
        // into fingerprints.
        //
        // The second layer is used to determine whether the fingerprint we just derived matches
        // any fingerprint from the target directory. If so, we add the *original* fingerprint
        // to the duplicates grouping map above.
        if let Ok(candidate_fingerprint) = LazyFingerprint::try_from_path(&item) {
            if let Some(fingerprint) = set.get(&candidate_fingerprint) {
                duplicates_grouping
                    .entry(fingerprint)
                    .or_default()
                    .push(item);
            }
        }
    }

    let groups = duplicates_grouping.into_iter().filter(|x| x.1.len() > 1);
    for (canonical, duplicates) in groups {
        println!("{}", canonical.path().display());
        duplicates
            .into_iter()
            .for_each(|x| println!("    {}", x.display()));
    }
}

fn materialize_target_tree(path: &Path, pwd: &Path) -> HashSet<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|x| x.ok().and_then(|x| fs::canonicalize(x.path()).ok()))
        .map(|x| x.strip_prefix(pwd).unwrap().into())
        .collect()
}

fn materialize_context_tree<'a>(
    path: &Path,
    target_tree: &'a HashSet<PathBuf>,
    pwd: &'a Path,
) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|x| x.ok().and_then(|x| fs::canonicalize(x.path()).ok()))
        .map(|x| x.strip_prefix(pwd).unwrap().into())
        .filter(|x| !target_tree.contains(x))
        .collect()
}
