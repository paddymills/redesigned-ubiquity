
USE SNDBase91;

IF EXISTS (SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'HighSteel' AND TABLE_NAME = 'RuntimeInfo') DROP TABLE HighSteel.RuntimeInfo
IF NOT EXISTS (SELECT name FROM sys.schemas WHERE name = 'HighSteel') DROP SCHEMA HighSteel;
GO

CREATE SCHEMA HighSteel;
GO

CREATE TABLE HighSteel.RuntimeInfo (
	id int IDENTITY(1,1) PRIMARY KEY,
	name varchar(255) ,
	last_runtime datetime
);
GO
