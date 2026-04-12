use crate::error::Result;
use crate::{git, print, ui};

pub fn release_tag() -> Result<()> {
    let previous = git::latest_release_tag()?;

    print::blank();
    print::header("Create release tag");
    if let Some(ref tag) = previous {
        print::hint(&format!("Previous release tag: {tag}"));
    } else {
        print::hint("Previous release tag: none");
    }
    print::blank();

    let tag = ui::prompt_release_tag(previous.as_deref())?;

    git::create_tag(&tag)?;
    git::push_tag(&tag)?;

    print::blank();
    print::success_with_details("Released", &format!("tag {tag} pushed"));
    print::blank();
    Ok(())
}
