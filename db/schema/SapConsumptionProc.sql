
USE SNDBase91;

IF EXISTS (SELECT name FROM sys.views WHERE name = 'SapConsumptionData') DROP VIEW SapConsumptionData;
IF EXISTS (SELECT name FROM sys.procedures WHERE name = 'SapProductionData') DROP PROCEDURE SapProductionData;
IF EXISTS (SELECT name FROM sys.procedures WHERE name = 'SapIssueData') DROP PROCEDURE SapIssueData;
IF EXISTS (SELECT name FROM sys.procedures WHERE name = 'SapProductionData_SinceLastRun') DROP PROCEDURE SapProductionData_SinceLastRun;
IF EXISTS (SELECT name FROM sys.procedures WHERE name = 'SapIssueData_SinceLastRun') DROP PROCEDURE SapIssueData_SinceLastRun;
GO

CREATE VIEW SapConsumptionData AS
	WITH
		Parts AS (
			SELECT
				ArchivePacketID,
				AutoId as Id,
				CASE WHEN Data1 LIKE REPLICATE('[0-9]', 7) + '[a-zA-Z]' -- regex: [0-9]{7}[a-zA-Z]
					THEN LEFT(Data1, 7)
					ELSE Data1
				END AS Job,
				CASE ISNUMERIC(Data2)
					WHEN 1
						THEN FORMAT(CONVERT(int,Data2), '00')
						ELSE Data2
				END AS Shipment,
				CASE Data3
					WHEN ''
						THEN REPLACE(PartName, '_', '-') -- fallback to part name
						ELSE Data3
				END AS PartName,
				Data4 AS PartWbs,
				QtyProgram AS Qty,
				NestedArea AS AreaPerEach
			FROM PartArchive
		),
		Sheets AS (
			SELECT
				ArchivePacketID,
				PrimeCode AS Material,
				Mill as Wbs,
				Location
			FROM StockArchive
		),
		Programs AS (
			SELECT
				ArchivePacketID,
				ArcDateTime,
				CASE LEFT(MachineName,7)
					WHEN 'Plant_3'
						THEN 'HS02'
						ELSE 'HS01'
				END AS Plant,
				ProgramName
			FROM ProgArchive
			WHERE TransType = 'SN102'
		)

	SELECT
		ArcDateTime,
		Id,
		PartName,
		Job,
		Shipment,
		PartWbs,
		'PROD' AS PartLocation,
		Parts.Qty AS PartQty,
		'EA' AS PartUoM,
		Sheets.Material AS MaterialMaster,
		Sheets.Wbs AS MaterialWbs,
		ROUND(Parts.AreaPerEach * Parts.Qty, 3) AS TotalNestedArea,
		'IN2' AS MaterialUoM,
		Sheets.Location AS MaterialLocation,
		Programs.Plant,
		Programs.ProgramName
	FROM Parts
		INNER JOIN Sheets
			ON Parts.ArchivePacketID=Sheets.ArchivePacketID
		INNER JOIN Programs
			ON Parts.ArchivePacketID=Programs.ArchivePacketID;
GO

CREATE PROCEDURE SapProductionData
	@Start DATETIME, @End DATETIME
AS
	DECLARE @WbsPattern varchar(64) = 'D-' + REPLICATE('[0-9]', 7) + '-' + REPLICATE('[0-9]', 5); -- regex: D-\d{7}-\d{5}

	SELECT
		PartName,
		Id,
		PartWbs,
		PartLocation,
		PartQty,
		PartUoM,
		MaterialMaster,
		MaterialWbs,
		TotalNestedArea,
		MaterialUoM,
		MaterialLocation,
		Plant,
		ProgramName
	FROM SapConsumptionData
	WHERE ArcDateTime >= @Start AND ArcDateTime < @End
	AND PartWbs LIKE @WbsPattern
	ORDER BY ProgramName, PartName;
GO

CREATE PROCEDURE SapIssueData
	@Start DATETIME, @End DATETIME
AS
	DECLARE @WbsPattern varchar(64) = 'D-' + REPLICATE('[0-9]', 7) + '-' + REPLICATE('[0-9]', 5); -- regex: D-\d{7}-\d{5}

	SELECT
		CASE
			WHEN Shipment LIKE '20[0-9][0-9]' THEN
				CASE MaterialWbs WHEN ''
					THEN 'CC01'
					ELSE 'CC02'
				END
			WHEN MaterialWbs LIKE 'D-' + Job + '-%'
				THEN 'PR01'
			WHEN MaterialWbs = ''
				THEN 'PR02'
			ELSE 'PR03'	-- Sheets.Wbs is for a different Job
		END AS Code,
		CASE
			WHEN Shipment LIKE '20[0-9][0-9]'
				THEN Shipment
				ELSE 'D-' + Job
		END AS User1,
		CASE WHEN Shipment LIKE '20[0-9][0-9]' THEN
			CASE WHEN
				-- wish there was a better way to do this,
				--   but this is the same as the regex `(^|[_-])machine($|[_-])` for each machine name
				PartName LIKE 'gemini[-_]%' OR PartName LIKE '%[-_]gemini' OR PartName LIKE '%[-_]gemini[-_]%' OR
				PartName LIKE 'titan[-_]%'  OR PartName LIKE '%[-_]titan'  OR PartName LIKE '%[-_]titan[-_]%'  OR
				PartName LIKE 'mg[-_]%'     OR PartName LIKE '%[-_]mg'     OR PartName LIKE '%[-_]mg[-_]%'     OR
				PartName LIKE 'farley[-_]%' OR PartName LIKE '%[-_]farley' OR PartName LIKE '%[-_]farley[-_]%' OR
				PartName LIKE 'ficep[-_]%'  OR PartName LIKE '%[-_]ficep'  OR PartName LIKE '%[-_]ficep[-_]%'

				THEN '634124' -- machine parts
				ELSE '637118' -- shop supplies
			END

			ELSE Shipment
		END AS User2,
		MaterialMaster,
		MaterialWbs,
		TotalNestedArea,
		MaterialUoM,
		MaterialLocation,
		Plant,
		Id
	FROM SapConsumptionData
	WHERE ArcDateTime >= @Start AND ArcDateTime < @End
	AND PartWbs NOT LIKE @WbsPattern
	ORDER BY ProgramName, PartName;
GO

CREATE PROCEDURE SapProductionData_SinceLastRun
	@End DATETIME
AS
	DECLARE @LastRun DATETIME = (SELECT last_runtime FROM HighSteel.RuntimeInfo WHERE name='SapProductionData');

	EXEC SapProductionData @Start = @LastRun, @End = @End;
GO

CREATE PROCEDURE SapIssueData_SinceLastRun
	@End DATETIME
AS
	DECLARE @LastRun DATETIME = (SELECT last_runtime FROM HighSteel.RuntimeInfo WHERE name='SapIssueData');

	EXEC SapIssueData @Start = @LastRun, @End = @End;
GO

IF NOT EXISTS (SELECT name FROM HighSteel.RuntimeInfo WHERE name='SapProductionData')
	INSERT INTO HighSteel.RuntimeInfo (name, last_runtime) VALUES ('SapProductionData', DATEADD(HOUR, DATEDIFF(HOUR, 0, GETDATE()), 0));
IF NOT EXISTS (SELECT name FROM HighSteel.RuntimeInfo WHERE name='SapIssueData')
	INSERT INTO HighSteel.RuntimeInfo (name, last_runtime) VALUES ('SapIssueData', DATEADD(HOUR, DATEDIFF(HOUR, 0, GETDATE()), 0));
GO
