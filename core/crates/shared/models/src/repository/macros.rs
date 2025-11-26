#[macro_export]
macro_rules! impl_repository_base {
    ($type:ty, $pool:ty, $pool_field:ident) => {
        impl systemprompt_traits::Repository for $type {
            type Pool = $pool;
            type Error = systemprompt_traits::RepositoryError;

            fn pool(&self) -> &Self::Pool {
                &self.$pool_field
            }
        }
    };
}

#[macro_export]
macro_rules! repository_query {
    ($pool:expr, $query:expr) => {
        sqlx::query($query).fetch_all($pool.pool()).await
    };
    ($pool:expr, $query:expr, $($param:expr),+) => {
        {
            let mut q = sqlx::query($query);
            $(
                q = q.bind($param);
            )+
            q.fetch_all($pool.pool()).await
        }
    };
}

#[macro_export]
macro_rules! repository_execute {
    ($pool:expr, $query:expr) => {
        sqlx::query($query).execute($pool.pool()).await
    };
    ($pool:expr, $query:expr, $($param:expr),+) => {
        {
            let mut q = sqlx::query($query);
            $(
                q = q.bind($param);
            )+
            q.execute($pool.pool()).await
        }
    };
}
