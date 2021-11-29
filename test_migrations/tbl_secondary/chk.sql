select count(*)
from BuildliteDb.sqlite_master
where Name = 'Secondary'
and Type = 'table';
