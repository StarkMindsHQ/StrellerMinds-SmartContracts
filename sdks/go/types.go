package strellerminds

// LearningSession represents a student's learning session
type LearningSession struct {
	ID        string `json:"id"`
	Student   string `json:"student"`
	StartTime uint64 `json:"start_time"`
	CourseID  string `json:"course_id"`
}

// ProgressAnalytics represents student progress data
type ProgressAnalytics struct {
	CompletedModules uint32 `json:"completed_modules"`
	TotalTime        uint64 `json:"total_time"`
	Score            uint32 `json:"score"`
}
