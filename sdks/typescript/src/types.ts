export interface LearningSession {
    id: string;
    student: string;
    startTime: number;
    courseId: string;
}

export interface ProgressAnalytics {
    completedModules: number;
    totalTime: number;
    score: number;
}
