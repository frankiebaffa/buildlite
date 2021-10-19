# BuildLite

A SQL builder for [wORM](https://github.com/frankiebaffa/worm).

_ _ _

## Useage

You should first look at the implementation details of [wORM](https://github.com/frankiebaffa/worm) and [wORM Derive](https://github.com/frankiebaffa/worm_derive).

```rust
use {
	buildlite::Query,
	worm_derive::Worm,
};
#[derive(Worm)]
#[dbmodel(table(schema="TestDb", name="TestTable", alias="testtable"))]
struct TestTable {
	#[dbcolumn(column(name="Id", primary_key))]
	id: i64,
	#[dbcolumn(column(name="Name", unique_name, insertable))]
	name: String,
	#[dbcolumn(column(name="Active", active_flag, insertable))]
	active: bool,
}
#[derive(Worm)]
#[dbmodel(table(schema="TestDb", name="AnotherTable", alias="anothertable"))]
struct AnotherTable {
	#[dbcolumn(column(name="Id", primary_key))]
	id: i64,
	#[dbcolumn(column(name="Test_Id", foreign_key="TestTable"))]
	test_id: i64,
	#[dbcolumn(column(name="Name", unique_name, insertable))]
	name: String,
	#[dbcolumn(column(name="Active", active_flag, insertable))]
	active: bool,
}
fn test_select() {
	let q = Query::<TestTable>::select()
		.where_eq::<TestTable>(TestTable::ID, &1).and()
		.where_gt::<TestTable>(TestTable::ACTIVE, &0);
	let test_against = format!(
		"select testtable.* from TestDb.TestTable as testtable where testtable.Id = :param0 and testtable.Active > :param1"
	);
	assert_eq!(q.query_to_string(), test_against);
}
```
