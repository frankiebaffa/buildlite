create table BuildliteDb.Secondary
	(
		Id integer primary key autoincrement
	,	Item_Id integer not null
	,	Name text unique not null
	,	foreign key (Item_Id) references Item (Id)
	);
