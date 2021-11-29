select count(*)
from BuildliteDb.sqlite_master
where Name = 'Item'
and Type = 'table';
