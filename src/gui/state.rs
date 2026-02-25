#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Show,
    Update,
    Watch,
    KubeTrack,
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Show => "Packages",
            Page::Update => "Update",
            Page::Watch => "Watch",
            Page::KubeTrack => "Kube Track",
        }
    }

    pub fn all() -> &'static [Page] {
        &[Page::Show, Page::Update, Page::Watch, Page::KubeTrack]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_labels() {
        assert_eq!(Page::Show.label(), "Packages");
        assert_eq!(Page::Update.label(), "Update");
        assert_eq!(Page::Watch.label(), "Watch");
        assert_eq!(Page::KubeTrack.label(), "Kube Track");
    }

    #[test]
    fn test_page_all_contains_all_variants() {
        let all = Page::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Page::Show));
        assert!(all.contains(&Page::Update));
        assert!(all.contains(&Page::Watch));
        assert!(all.contains(&Page::KubeTrack));
    }

    #[test]
    fn test_page_default_is_show() {
        assert_eq!(Page::default(), Page::Show);
    }

    #[test]
    fn test_page_equality() {
        assert_eq!(Page::Show, Page::Show);
        assert_ne!(Page::Show, Page::Update);
    }
}
