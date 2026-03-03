pub trait RenderTarget {}
pub trait RenderEnvironment {}

pub trait Renderer<T: RenderTarget, E: RenderEnvironment> {
    fn render(&self) -> anyhow::Result<String>;
}

pub struct Latex;
pub struct Html;
pub struct Markdown;

impl RenderTarget for Latex {}
impl RenderTarget for Html {}
impl RenderTarget for Markdown {}

pub struct Problem;
pub struct Solution;
pub struct Universal;

impl RenderEnvironment for Problem {}
impl RenderEnvironment for Solution {}
impl RenderEnvironment for Universal {}

impl<R, T> Renderer<T, Problem> for R
where
    R: Renderer<T, Universal>,
    T: RenderTarget
{
    fn render(&self) -> anyhow::Result<String> {
        <R as Renderer<T, Universal>>::render(self)
    }
}

impl<R, T> Renderer<T, Solution> for R
where
    R: Renderer<T, Universal>,
    T: RenderTarget
{
    fn render(&self) -> anyhow::Result<String> {
        <R as Renderer<T, Universal>>::render(self)
    }
}
