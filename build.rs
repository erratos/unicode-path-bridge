fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile("src/eupb.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
