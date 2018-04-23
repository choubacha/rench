pub enum Message<T>
where
    T: 'static + Send,
{
    Body(T),
    EOF,
}
