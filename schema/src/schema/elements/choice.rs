use crate::{
    Element, Html, Image, Latex, List, Markdown, OrderFormat, OrderType, Paragraph, Renderer, Text,
    Universal,
};

#[derive(Debug, Clone)]
pub enum Choice {
    Text(Text),
    Image(Image),
}

#[derive(Debug, Clone)]
pub struct ChoicePool {
    pub choices: Vec<Choice>,
    pub order_type: OrderType,
    pub order_format: OrderFormat,
}

impl TryFrom<List> for ChoicePool {
    type Error = String;

    fn try_from(value: List) -> Result<Self, Self::Error> {
        let choices = value
            .items
            .into_iter()
            .map(|item| {
                if item.elements.len() != 1 {
                    return Err("each list item must contain exactly one element".to_string());
                }
                let element = item.elements.first().unwrap();
                match element {
                    Element::Text(text) => Ok(Choice::Text(text.clone())),
                    Element::Image(image) => Ok(Choice::Image(image.clone())),
                    _ => Err("list items must be either text or image elements".to_string()),
                }
            })
            .collect::<Result<Vec<Choice>, String>>()?;

        Ok(ChoicePool {
            choices,
            order_type: value.order_type,
            order_format: value.order_format,
        })
    }
}

impl From<ChoicePool> for List {
    fn from(value: ChoicePool) -> Self {
        let items = value
            .choices
            .into_iter()
            .map(|choice| {
                let element = match choice {
                    Choice::Text(text) => Element::Text(text),
                    Choice::Image(image) => Element::Image(image),
                };
                Paragraph::new(vec![element])
            })
            .collect();

        List {
            items,
            order_type: value.order_type,
            order_format: value.order_format,
        }
    }
}

impl Renderer<Latex, Universal> for ChoicePool {
    fn render(&self) -> anyhow::Result<String> {
        let list: List = self.clone().into();
        <List as Renderer<Latex, Universal>>::render(&list)
    }
}

impl Renderer<Html, Universal> for ChoicePool {
    fn render(&self) -> anyhow::Result<String> {
        let list: List = self.clone().into();
        <List as Renderer<Html, Universal>>::render(&list)
    }
}

impl Renderer<Markdown, Universal> for ChoicePool {
    fn render(&self) -> anyhow::Result<String> {
        let list: List = self.clone().into();
        <List as Renderer<Markdown, Universal>>::render(&list)
    }
}
