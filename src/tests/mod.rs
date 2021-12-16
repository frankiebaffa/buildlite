mod query_builder {
    use {
        crate::Query,
        worm::derive::Worm,
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
    #[test]
    fn test_select() {
        let q = Query::<TestTable>::select()
            .where_eq::<TestTable>(TestTable::ID, &1).and()
            .where_gt::<TestTable>(TestTable::ACTIVE, &0);
        let test_against = format!(
            "select testtable.* from TestDb.TestTable as testtable where testtable.Id = :param0 and testtable.Active > :param1"
        );
        assert_eq!(q.query_to_string(), test_against);
    }
    #[test]
    fn test_join() {
        let q = Query::<TestTable>::select()
            .join::<AnotherTable>().join_and()
            .join_eq::<AnotherTable>(AnotherTable::ACTIVE, &1)
            .where_eq::<TestTable>(TestTable::ID, &1);
        let test_against = format!(
            "select testtable.* from TestDb.TestTable as testtable join TestDb.AnotherTable as anothertable on testtable.Id = anothertable.Test_Id and anothertable.Active = :param0 where testtable.Id = :param1"
        );
        assert_eq!(q.query_to_string(), test_against);
    }
    #[test]
    fn test_join_fk() {
        let q = Query::<AnotherTable>::select()
            .join_fk::<TestTable>().join_and()
            .join_fk_eq::<TestTable>(TestTable::ID, &1)
            .where_eq::<AnotherTable>(AnotherTable::ACTIVE, &1);
        let test_against = format!(
            "select anothertable.* from TestDb.AnotherTable as anothertable join TestDb.TestTable as testtable on anothertable.Test_Id = testtable.Id and testtable.Id = :param0 where anothertable.Active = :param1"
        );
        assert_eq!(q.query_to_string(), test_against);
    }
    #[test]
    fn test_orderby_asc() {
        let q = Query::<AnotherTable>::select()
            .orderby_desc(AnotherTable::NAME)
            .where_eq::<AnotherTable>(AnotherTable::ACTIVE, &1);
        let test_against = format!(
            "select anothertable.* from TestDb.AnotherTable as anothertable where anothertable.Active = :param0 order by anothertable.Name desc"
        );
        assert_eq!(q.query_to_string(), test_against);
    }
}
mod execution {
    use {
        crate::Query,
        migaton::traits::{
            DoMigrations,
            Migrations,
        },
        serial_test::serial,
        worm::{
            core::{
                DbContext,
                DbCtx,
                ForeignKey,
                PrimaryKey,
            },
            derive::{
                Worm,
                WormDb,
            },
        },
    };
    #[derive(WormDb)]
    #[db(var(name="BUILDLITEDB"))]
    struct Database {
        context: DbContext,
    }
    pub struct BuildliteMigrator;
    impl BuildliteMigrator {
        const MIGRATIONS_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_migrations");
    }
    impl Migrations for BuildliteMigrator {
        fn get_mig_path() -> &'static str {
            return Self::MIGRATIONS_PATH;
        }
    }
    #[derive(Worm)]
    #[dbmodel(table(schema="BuildliteDb", name="Item", alias="item"))]
    struct Item {
        #[dbcolumn(column(name="Id", primary_key))]
        id: i64,
        #[dbcolumn(column(name="Name", insertable))]
        name: String,
    }
    #[derive(Worm)]
    #[dbmodel(table(schema="BuildliteDb", name="Secondary", alias="secondary"))]
    struct Secondary {
        #[dbcolumn(column(name="Id", primary_key))]
        id: i64,
        #[dbcolumn(column(name="Item_Id", foreign_key="Item", insertable))]
        item_id: i64,
        #[dbcolumn(column(name="Name", insertable))]
        name: String,
    }
    fn get_db_ctx() -> (Database, Database) {
        let mut mem_db = Database::init();
        mem_db.context.attach_temp_dbs();
        let mut db = Database::init();
        db.context.attach_dbs();
        return (mem_db, db);
    }
    fn migrate_up(mem_db: &mut Database, db: &mut Database) {
        BuildliteMigrator::migrate_up(mem_db, db);
    }
    fn migrate_down(mem_db: &mut Database, db: &mut Database) {
        BuildliteMigrator::migrate_down(mem_db, db);
    }
    #[test]
    #[serial]
    fn migrations() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        migrate_down(&mut mem_db, &mut db);
    }
    const PRIMARY_NAME: &'static str = "Hello";
    fn new_item(db: &mut Database) -> Item {
        let name = PRIMARY_NAME;
        let i_res = Item::insert_new(db, name.to_string());
        assert!(i_res.is_ok());
        let i = i_res.unwrap();
        assert_eq!(i.get_name(), name);
        return i;
    }
    #[test]
    #[serial]
    fn insert_item() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        new_item(&mut db);
        migrate_down(&mut mem_db, &mut db);
    }
    fn select_from_item(db: &mut Database) -> Item {
        let p = new_item(db);
        let q_res = Query::<Item>::select()
            .where_eq::<Item>(Item::NAME, &PRIMARY_NAME)
            .execute_row(db);
        assert!(q_res.is_ok());
        let q = q_res.unwrap();
        assert_eq!(p.get_id(), q.get_id());
        assert_eq!(p.get_name(), q.get_name());
        return q;
    }
    #[test]
    #[serial]
    fn select_item() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        select_from_item(&mut db);
        migrate_down(&mut mem_db, &mut db);
    }
    const SECONDARY_NAME: &'static str = "World";
    fn new_secondary(db: &mut Database, p: &Item) -> Secondary {
        let s_res = Secondary::insert_new(db, p.get_id(), SECONDARY_NAME.to_string());
        assert!(s_res.is_ok());
        let s = s_res.unwrap();
        assert_eq!(s.get_name(), SECONDARY_NAME);
        assert_eq!(s.get_fk_value(), p.get_id());
        return s;
    }
    #[test]
    #[serial]
    fn insert_secondary() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        let p = new_item(&mut db);
        new_secondary(&mut db, &p);
        migrate_down(&mut mem_db, &mut db);
    }
    fn select_from_secondary(db: &mut Database, p: &Item) -> Secondary {
        let q_res = Query::<Secondary>::select()
            .where_eq::<Secondary>(Secondary::NAME, &SECONDARY_NAME)
            .execute_row(db);
        assert!(q_res.is_ok());
        let q = q_res.unwrap();
        assert_eq!(q.get_name(), SECONDARY_NAME);
        assert_eq!(q.get_fk_value(), p.get_id());
        return q;
    }
    #[test]
    #[serial]
    fn select_secondary() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        let p = new_item(&mut db);
        new_secondary(&mut db, &p);
        select_from_secondary(&mut db, &p);
        migrate_down(&mut mem_db, &mut db);
    }
    fn join_on_secondary(db: &mut Database, p: &Item, s: &Secondary) -> Secondary {
        let pid = p.get_id();
        let q_res = Query::<Secondary>::select()
            .join_fk::<Item>().join_and()
            .join_fk_eq::<Item>(Item::ID, &pid)
            .execute_row(db);
        assert!(q_res.is_ok());
        let q = q_res.unwrap();
        assert_eq!(q.get_id(), s.get_id());
        assert_eq!(q.get_fk_value(), p.get_id());
        assert_eq!(q.get_name(), s.get_name());
        return q;
    }
    #[test]
    #[serial]
    fn join_secondary() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        let p = new_item(&mut db);
        new_secondary(&mut db, &p);
        let s = select_from_secondary(&mut db, &p);
        join_on_secondary(&mut db, &p, &s);
        migrate_down(&mut mem_db, &mut db);
    }
    fn join_on_item(db: &mut Database, p: &Item, s: &Secondary) -> Item {
        let q_res = Query::<Item>::select()
            .join::<Secondary>().join_and()
            .join_eq::<Secondary>(Secondary::ID, &s.get_id())
            .execute_row(db);
        assert!(q_res.is_ok());
        let q = q_res.unwrap();
        assert_eq!(q.get_id(), p.get_id());
        assert_eq!(q.get_name(), p.get_name());
        return q;
    }
    #[test]
    #[serial]
    fn join_item() {
        let (mut mem_db, mut db) = get_db_ctx();
        migrate_up(&mut mem_db, &mut db);
        let p = new_item(&mut db);
        new_secondary(&mut db, &p);
        let s = select_from_secondary(&mut db, &p);
        join_on_item(&mut db, &p, &s);
        migrate_down(&mut mem_db, &mut db);
    }
}
