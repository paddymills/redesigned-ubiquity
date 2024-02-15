

USE SNDBase91;

IF EXISTS (SELECT name FROM sys.procedures WHERE name = 'GetProgramStatus') DROP PROCEDURE GetProgramStatus;
GO

CREATE PROCEDURE GetProgramStatus
	@ProgramName VARCHAR(255)
AS
	IF EXISTS (SELECT TOP 1 1 FROM Program WHERE ProgramName = @ProgramName)
		-- Active Program (Posted, not updated)
		BEGIN
			SELECT
				'Active' AS Status,
				Program.ProgramName,
				Program.PostDateTime AS Timestamp,
				Stock.SheetName,
				Stock.PrimeCode AS MaterialMaster
			FROM Program
			INNER JOIN Stock
				ON Stock.SheetName = Program.SheetName
			WHERE Program.ProgramName = @ProgramName
		END
	ELSE
		-- Updated or Deleted programs
		BEGIN
			SELECT TOP 1
				CASE TransType
					WHEN 'SN102' THEN 'Updated'
					WHEN 'SN101' THEN 'Deleted'
				END AS Status,
				Program.ProgramName,
				CASE WHEN CompletedProgram.CompletedDateTime IS NOT NULL
					THEN CompletedProgram.CompletedDateTime
					ELSE Program.ArcDateTime
				END AS Timestamp,
				Program.ProgramName,
				Stock.SheetName,
				Stock.PrimeCode AS MaterialMaster,
				Stock.HeatNumber,
				LEFT(Stock.BinNumber, 10) AS PoNumber,
				NULLIF(Stock.Mill, '') AS Wbs,
				NULLIF(CompletedProgram.OperatorName, '') AS Operator
			FROM ProgArchive AS Program
			LEFT OUTER JOIN StockArchive AS Stock
				ON Stock.ArchivePacketID = Program.ArchivePacketID
			LEFT OUTER JOIN CompletedProgram
				ON Program.ProgramName = CompletedProgram.ProgramName
				AND Program.SheetName = CompletedProgram.SheetName
			WHERE Program.ProgramName = @ProgramName
			-- select newest record with TransType='SN102' if it exists,
			--  else select the newest record with TransType='SN101'
			AND Program.TransType =
				CASE WHEN EXISTS (SELECT TOP 1 1 FROM ProgArchive WHERE ProgramName = @ProgramName AND TransType = 'SN102')
					THEN 'SN102'
					ELSE 'SN101'
				END
			ORDER BY Program.ArcDateTime DESC
		END
