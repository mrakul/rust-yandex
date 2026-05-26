use std::sync::Arc;
use tracing::instrument;

use crate::domain::{post::Post, BlogError};
use crate::data::post_repository::PostRepository;

#[derive(Clone)]
pub struct BlogService<R: PostRepository + 'static> {
    repo: Arc<R>,
}

impl<R> BlogService<R>
where
    R: PostRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_post(&self, author_id: i64, title: String, content: String) -> Result<Post, BlogError> {
        if title.is_empty() {
            return Err(BlogError::Validation("Пустой заголовок поста".into()));
        }
        if content.is_empty() {
            return Err(BlogError::Validation("Пустой контент поста".into()));
        }

        let post = Post::new(author_id, title, content);
        self.repo.create(post).await.map_err(BlogError::from)
    }

    #[instrument(skip(self))]
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogError>
    {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| BlogError::NotFound(format!("Пост {}", id)))
    }

    #[instrument(skip(self))]
    pub async fn update_post(&self, id: i64, author_id: i64, title: Option<String>, content: Option<String>) -> Result<Post, BlogError> {
        let mut post = self.get_post(id).await?;
        
        // Проверка автора
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }

        // Обновляем пост и запись
        post.update(title, content);
        self.repo.update(post.clone()).await.map_err(BlogError::from)?;
        
        Ok(post)
    }

    #[instrument(skip(self))]
    pub async fn delete_post(&self, id: i64, author_id: i64) -> Result<(), BlogError> {
        let post = self.get_post(id).await?;
        
        // Проверка автора
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }

        self.repo.delete(id).await.map_err(BlogError::from)
    }

    #[instrument(skip(self))]
    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), BlogError> 
    {
        // Ограничиваем limit, offset 0 по умолчанию
        let limit = limit.min(100).max(1);
        let posts = self.repo.list(limit, offset).await.map_err(BlogError::from)?;
        let total = self.repo.count().await.map_err(BlogError::from)?;
    
        Ok((posts, total))
    }
}