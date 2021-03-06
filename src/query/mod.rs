use {
    crate::error::{
        BuildliteError,
        BuildliteErrorMatch,
    },
    std::collections::HashMap,
    worm::core::{
        DbCtx,
        ForeignKey,
        PrimaryKeyModel,
        sql::ToSql,
    },
};
enum QueryType {
    Select,
    Update,
}
pub struct Query<'query, T> {
    query_type: QueryType,
    select: String,
    update: String,
    set: Option<String>,
    from: String,
    join: Option<String>,
    clause: Option<String>,
    orderby: Option<String>,
    _value: Option<T>,
    select_params: HashMap<String, Box<&'query dyn ToSql>>,
    update_params: HashMap<String, Box<&'query dyn ToSql>>,
}
impl<'query, T> Query<'query, T> where T: PrimaryKeyModel {
    pub fn select() -> Self {
        return Query {
            query_type: QueryType::Select,
            select: format!("select {}.*", T::ALIAS),
            update: String::new(),
            set: None,
            from: format!("from {}.{} as {}", T::DB, T::TABLE, T::ALIAS),
            join: None,
            clause: None,
            orderby: None,
            _value: None,
            select_params: HashMap::new(),
            update_params: HashMap::new(),
        };
    }
    pub fn update() -> Self {
        return Query {
            query_type: QueryType::Update,
            select: format!("select {}.*", T::ALIAS),
            update: format!("update {}.{}", T::DB, T::TABLE),
            set: None,
            from: String::new(),
            join: None,
            clause: None,
            orderby: None,
            _value: None,
            select_params: HashMap::new(),
            update_params: HashMap::new(),
        };
    }
    pub fn set<'a>(mut self, column: &'a str, value: &'query dyn ToSql) -> Self {
        let dlim;
        let set;
        if self.set.is_none() {
            set = String::new();
            dlim = "set ";
        } else {
            set = self.set.unwrap();
            dlim = ", ";
        }
        let param_num = self.select_params.len() + self.update_params.len();
        let param_name = format!(":param{}", param_num);
        self.update_params.insert(param_name.clone(), Box::new(value));
        self.set = Some(format!(
            "{}{}{} = {}",
            set, dlim, column, param_name
        ));
        return self;
    }
    pub fn join_fk<U>(mut self) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>,
    {
        match self.query_type {
            QueryType::Select => {},
            QueryType::Update => panic!("Update-From is not yet supported"),
        }
        let join_str;
        let dlim;
        if self.join.is_none() {
            join_str = String::new();
            dlim = String::new();
        } else {
            join_str = self.join.unwrap();
            dlim = String::from(" ");
        }
        self.join = Some(
            format!(
                "{}{}join {}.{} as {} on {}.{} = {}.{}",
                join_str, dlim,
                U::DB, U::TABLE, U::ALIAS,
                T::ALIAS, T::FOREIGN_KEY, U::ALIAS, U::PRIMARY_KEY,
            )
        );
        return self;
    }
    fn filter_join_fk<'a, U>(
        mut self,
        op: &'a str,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>,
    {
        match self.query_type {
            QueryType::Select => {},
            QueryType::Update => panic!("Update-From is not yet supported"),
        }
        let join_str;
        let dlim;
        if self.join.is_none() {
            panic!("Cannot add another join constraint when there is no join");
        } else {
            join_str = self.join.unwrap();
            dlim = String::from(" ");
        }
        let param_num = self.select_params.len() + self.update_params.len();
        let param_name = format!(":param{}", param_num);
        self.select_params.insert(param_name.clone(), Box::new(value));
        self.join = Some(
            format!(
                "{}{}{}.{} {} {}",
                join_str, dlim,
                U::ALIAS, column,
                op, param_name,
            )
        );
        return self;
    }
    pub fn join_fk_eq<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>("=", column, value);
    }
    pub fn join_fk_ne<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>("!=", column, value);
    }
    pub fn join_fk_gt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>(">", column, value);
    }
    pub fn join_fk_lt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>("<", column, value);
    }
    pub fn join_fk_ge<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>(">=", column, value);
    }
    pub fn join_fk_le<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel,
        T: ForeignKey<U>
    {
        return self.filter_join_fk::<U>("<=", column, value);
    }
    pub fn join<U>(mut self) -> Self
    where
        U: ForeignKey<T>
    {
        match self.query_type {
            QueryType::Select => {},
            QueryType::Update => panic!("Update-From is not yet supported"),
        }
        let join_str;
        let dlim;
        if self.join.is_none() {
            join_str = String::new();
            dlim = String::new();
        } else {
            join_str = self.join.unwrap();
            dlim = String::from(" ");
        }
        self.join = Some(
            format!(
                "{}{}join {}.{} as {} on {}.{} = {}.{}",
                join_str, dlim,
                U::DB, U::TABLE, U::ALIAS,
                T::ALIAS, T::PRIMARY_KEY, U::ALIAS, U::FOREIGN_KEY
            )
        );
        return self;
    }
    pub fn join_and(mut self) -> Self {
        match self.query_type {
            QueryType::Select => {},
            QueryType::Update => panic!("Update-From is not yet supported"),
        }
        let join_str;
        let dlim;
        if self.join.is_none() {
            panic!("Cannot concatenate a join when no join exists");
        } else {
            join_str = self.join.unwrap();
            dlim = String::from(" ");
        }
        self.join = Some(
            format!(
                "{}{}and",
                join_str, dlim
            )
        );
        return self;
    }
    fn filter_join<'a, U>(
        mut self,
        op: &'a str,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        match self.query_type {
            QueryType::Select => {},
            QueryType::Update => panic!("Update-From is not yet supported"),
        }
        let join_str;
        let dlim;
        if self.join.is_none() {
            panic!("Cannot add another join constraint when there is no join");
        } else {
            join_str = self.join.unwrap();
            dlim = String::from(" ");
        }
        let param_num = self.select_params.len() + self.update_params.len();
        let param_name = format!(":param{}", param_num);
        self.select_params.insert(param_name.clone(), Box::new(value));
        self.join = Some(
            format!(
                "{}{}{}.{} {} {}",
                join_str, dlim,
                U::ALIAS, column,
                op, param_name,
            )
        );
        return self;
    }
    pub fn join_eq<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>("=", column, value);
    }
    pub fn join_ne<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>("!=", column, value);
    }
    pub fn join_gt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>(">", column, value);
    }
    pub fn join_lt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>("<", column, value);
    }
    pub fn join_ge<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>(">=", column, value);
    }
    pub fn join_le<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: ForeignKey<T>
    {
        return self.filter_join::<U>("<=", column, value);
    }
    fn filter<'a, U>(
        mut self,
        op: &'a str,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        let clause_str;
        let dlim;
        if self.clause.is_none() {
            clause_str = String::new();
            dlim = String::from("where ");
        } else {
            clause_str = self.clause.unwrap();
            dlim = String::from(" ");
        }
        let param_num = self.select_params.len() + self.update_params.len();
        let param_name = format!(":param{}", param_num);
        self.select_params.insert(param_name.clone(), Box::new(value));
        match self.query_type {
            QueryType::Select => {
                self.clause = Some(
                    format!(
                        "{}{}{}.{} {} {}",
                        clause_str, dlim,
                        U::ALIAS, column,
                        op, param_name,
                    )
                );
            },
            QueryType::Update => {
                self.clause = Some(
                    format!(
                        "{}{}{} {} {}",
                        clause_str, dlim,
                        column,
                        op, param_name,
                    )
                );
            },
        }
        return self;
    }
    pub fn where_eq<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>("=", column, value);
    }
    pub fn where_ne<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>("!=", column, value);
    }
    pub fn where_gt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>(">", column, value);
    }
    pub fn where_lt<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>("<", column, value);
    }
    pub fn where_ge<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>(">=", column, value);
    }
    pub fn where_le<'a, U>(
        self,
        column: &'a str,
        value: &'query dyn ToSql
    ) -> Self
    where
        U: PrimaryKeyModel
    {
        return self.filter::<U>("<=", column, value);
    }
    fn concat<'a>(mut self, word: &'a str) -> Self {
        let clause_str;
        let dlim;
        if self.clause.is_none() {
            panic!("Cannot concatenate a clause when no clause exists");
        } else {
            clause_str = self.clause.unwrap();
            dlim = String::from(" ");
        }
        self.clause = Some(
            format!(
                "{}{}{}",
                clause_str, dlim, word
            )
        );
        return self;
    }
    pub fn and(self) -> Self {
        return self.concat("and");
    }
    pub fn or(self) -> Self {
        return self.concat("or");
    }
    fn order_by<'a>(
        mut self,
        dir: &'a str,
        column: &'a str,
    ) -> Self {
        let orderby_str;
        let dlim;
        if self.orderby.is_none() {
            orderby_str = String::new();
            dlim = String::from("order by ");
        } else {
            orderby_str = self.orderby.unwrap();
            dlim = String::from(", ");
        }
        match self.query_type {
            QueryType::Select => {
                self.orderby = Some(
                    format!(
                        "{}{}{}.{} {}",
                        orderby_str, dlim,
                        T::ALIAS, column,
                        dir,
                    )
                );
            },
            QueryType::Update => {
                panic!("Update-From not yet supported");
            },
        }
        return self;
    }
    pub fn orderby_asc<'a>(self, column: &'a str) -> Self {
        return self.order_by("asc", column);
    }
    pub fn orderby_desc<'a>(self, column: &'a str) -> Self {
        return self.order_by("desc", column);
    }
    pub fn query_to_string(&self) -> String {
        let mut sql;
        match self.query_type {
            QueryType::Select => {
                sql = format!("{} {}", self.select, self.from);
                if self.join.is_some() {
                    let join = self.join.clone().unwrap();
                    sql.push_str(&format!(" {}", join));
                }
            },
            QueryType::Update => {
                if self.set.is_none() {
                    panic!("Cannot create an update statement without any set values");
                }
                sql = format!("{} {}", self.update, self.set.clone().unwrap());
            },
        }
        if self.clause.is_some() {
            let clause = self.clause.clone().unwrap();
            sql.push_str(&format!(" {}", clause));
        }
        if self.orderby.is_some() {
            let orderby = self.orderby.clone().unwrap();
            sql.push_str(&format!(" {}", orderby));
        }
        return sql;
    }
    pub fn execute_update(self, db: &mut impl DbCtx) -> Result<usize, BuildliteError> {
        let mut sql = self.query_to_string();
        // get query order of parameters
        let keys;
        let select_keys = self.select_params.keys();
        match self.query_type {
            QueryType::Select => {
                panic!("Cannot execute an update from a select query");
            }
            QueryType::Update => {
                let update_keys = self.update_params.keys();
                keys = select_keys.chain(update_keys);
            },
        }
        let mut key_indices: Vec<(usize, String)> = Vec::new();
        for key in keys {
            let index = sql.find(key).unwrap();
            sql = sql.replace(key, "?");
            key_indices.push((index, key.clone()));
        }
        key_indices.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut value_order = Vec::new();
        for key_index in key_indices {
            let key = key_index.1;
            let value = match self.select_params.get(&key) {
                Some(value) => value,
                None => self.update_params.get(&key).unwrap(),
            };
            value_order.push(value);
        }
        let param = worm::core::sql::params_from_iter(value_order);
        let c = db.use_connection();
        match self.query_type {
            QueryType::Select => {
                panic!("Cannot execute an update on a select query");
            },
            QueryType::Update => {
                return Ok(c.execute(&sql, param).quick_match()?);
            },
        }
    }
    pub fn execute(self, db: &mut impl DbCtx) -> Result<Vec<T>, BuildliteError> {
        let mut sql = self.query_to_string();
        // get query order of parameters
        let keys;
        let select_keys = self.select_params.keys();
        match self.query_type {
            QueryType::Select => {
                keys = select_keys;
            }
            QueryType::Update => {
                panic!("Cannot execute a select on an update query");
            },
        }
        let mut key_indices: Vec<(usize, String)> = Vec::new();
        for key in keys {
            let index = sql.find(key).unwrap();
            sql = sql.replace(key, "?");
            key_indices.push((index, key.clone()));
        }
        key_indices.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut value_order = Vec::new();
        for key_index in key_indices {
            let key = key_index.1;
            let value = match self.select_params.get(&key) {
                Some(value) => value,
                None => self.update_params.get(&key).unwrap(),
            };
            value_order.push(value);
        }
        let param = worm::core::sql::params_from_iter(value_order);
        let c = db.use_connection();
        let mut objs = Vec::new();
        match self.query_type {
            QueryType::Select => {
                let mut stmt = c.prepare(&sql).quick_match()?;
                let mut rows = stmt.query(param).quick_match()?;
                while let Some(row) = rows.next().quick_match()? {
                    objs.push(T::from_row(row).quick_match()?);
                }
            },
            QueryType::Update => {
                panic!("Cannot execute a select on an update query");
            },
        }
        return Ok(objs);
    }
    pub fn execute_row(self, db: &mut impl DbCtx) -> Result<T, BuildliteError> {
        let res = self.execute(db)?;
        if res.len() == 0 {
            return Err(BuildliteError::NoRowsError);
        } else {
            let val = res.into_iter().nth(0).unwrap();
            return Ok(val);
        }
    }
}

