pub trait UpdateState {
    async fn update(&mut self);
}
