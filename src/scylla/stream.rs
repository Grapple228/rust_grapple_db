use async_trait::async_trait;
use charybdis::{model::Model, stream::CharybdisModelStream};
use futures::StreamExt;

use crate::Pagable;

pub struct PagableCharybdisStream<E>
where
    E: Model + 'static,
{
    stream: CharybdisModelStream<E>,
    per_page: i32,
    page_num: i32,
    page_items: Vec<E>,
}

impl<E> PagableCharybdisStream<E>
where
    E: Model + 'static,
{
    pub fn new(stream: CharybdisModelStream<E>, per_page: i32) -> Self {
        Self {
            stream,
            per_page,
            page_num: 0,
            page_items: Vec::with_capacity(per_page as usize),
        }
    }
}

#[async_trait]
impl<E> Pagable<E> for PagableCharybdisStream<E>
where
    E: Model + 'static + Send + Sync,
{
    async fn next_page(&mut self) -> Option<&[E]> {
        self.page_items.clear();
        let mut available = 0;

        for _ in 0..self.per_page {
            match self.stream.next().await {
                Some(Ok(item)) => {
                    self.page_items.push(item);
                    available += 1;
                }
                _ => break, // Если произошла ошибка или больше нет элементов, выходим из цикла
            }
        }

        if available == 0 {
            None
        } else {
            self.page_num += 1;
            Some(self.page_items())
        }
    }

    async fn skip_page(&mut self) {
        self.page_items.clear();

        for _ in 0..self.per_page {
            if self.stream.next().await.is_none() {
                break; // Если больше нет элементов, выходим из цикла
            }
        }

        self.page_num += 1; // Увеличиваем номер страницы, если пропустили хотя бы одну
    }

    fn page_num(&self) -> i32 {
        self.page_num
    }

    fn page_items(&self) -> &[E] {
        &self.page_items
    }
}
