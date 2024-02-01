CREATE PROCEDURE SapAnalysis_PrevWeek AS
	DECLARE @PrevWeekSunday DATETIME
	DECLARE @ThisWeekSunday DATETIME
	SELECT @PrevWeekSunday = DATEADD(wk, DATEDIFF(wk, 6, GETDATE()), -1)
	SELECT @ThisWeekSunday = DATEADD(wk, DATEDIFF(wk, 0, GETDATE()), -1)

	SELECT
		Id,
		ArcDateTime AS UpdateDate,
		PartName AS Part,
		ProgramName AS Program,
		PartQty AS Qty,
		TotalNestedArea AS Area,
		MaterialLocation AS Location,
		MaterialMaster,
		MaterialWbs AS Wbs,
		Plant,
		'' AS OrderOrDocument,
		'' AS SAPValue,
		'' AS Notes

	FROM SapConsumptionData
	WHERE ArcDateTime >= @PrevWeekSunday
	AND ArcDateTime < @ThisWeekSunday
	ORDER BY MaterialMaster, ProgramName, PartName