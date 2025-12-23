#[must_use] pub fn build_admin_analysis_prompt(focus_area: &str, time_period: &str) -> String {
    format!(
        "You are a SystemPrompt administrator analyzing the system. Focus on: {focus_area}\n\
        Time period: {time_period}\n\n\
        Use the following tools to gather comprehensive data:\n\
        1. get_logs - Review system logs for errors, warnings, and patterns\n\
        2. db_admin - Analyze database performance and structure\n\
        3. system_status - Check disk space, memory, and critical resources\n\
        4. user_activity - Analyze user engagement and activity patterns\n\n\
        Provide analysis in this structure:\n\
        ## Executive Summary\n\
        - Overall system health status\n\
        - Critical issues requiring immediate attention\n\
        - Performance trends\n\n\
        ## Detailed Analysis\n\
        ### Logs Analysis\n\
        - Error patterns and frequency\n\
        - Warning trends\n\
        - Performance indicators\n\n\
        ### Database Health\n\
        - Connection status and performance\n\
        - Table sizes and growth patterns\n\
        - Query performance indicators\n\n\
        ### System Resources\n\
        - Disk space utilization and trends\n\
        - Memory usage patterns\n\
        - Critical resource availability\n\n\
        ### User Activity\n\
        - Active user counts and trends\n\
        - Engagement patterns\n\
        - Growth metrics\n\n\
        ## Recommendations\n\
        - Immediate actions required\n\
        - Medium-term optimizations\n\
        - Long-term strategic considerations\n\n\
        Focus your analysis on the {focus_area} area with a {time_period} time horizon."
    )
}
