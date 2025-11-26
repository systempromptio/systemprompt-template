pub fn build_system_health_prompt(include_recommendations: bool) -> String {
    format!(
        "Perform a comprehensive SystemPrompt health check using all available admin tools.\n\n\
        Execute the following diagnostic sequence:\n\n\
        1. **System Status Check**\n\
        - Use system_status tool to check disk space, memory, and database connectivity\n\
        - Identify any resource constraints or critical thresholds\n\n\
        2. **Database Health Assessment**\n\
        - Use db_admin with action='info' to check database connectivity and version\n\
        - Use db_admin with action='tables' to review table structure and sizes\n\
        - Look for unusually large tables or growth patterns\n\n\
        3. **Log Analysis**\n\
        - Use get_logs with level_filter='error' to identify critical issues\n\
        - Use get_logs with level_filter='warn' to spot potential problems\n\
        - Analyze recent log patterns for anomalies\n\n\
        4. **User Activity Review**\n\
        - Use user_activity to assess system usage and growth\n\
        - Check for unusual activity patterns or user engagement drops\n\n\
        Provide your health assessment in this format:\n\n\
        # SystemPrompt Health Report\n\
        **Generated**: [Current timestamp]\n\
        **Status**: 🟢 HEALTHY / 🟡 WARNING / 🔴 CRITICAL\n\n\
        ## Resource Status\n\
        - **Disk Space**: [Available/Used with percentage]\n\
        - **Memory**: [Usage statistics]\n\
        - **Database**: [Connection status and performance]\n\n\
        ## Critical Issues\n\
        [List any immediate concerns requiring attention]\n\n\
        ## Warning Indicators\n\
        [List potential issues that should be monitored]\n\n\
        ## Performance Metrics\n\
        - **Recent Errors**: [Count from logs]\n\
        - **User Activity**: [Recent activity trends]\n\
        - **System Responsiveness**: [Database response times]\n\n\
        {}\n\n\
        **Next Review**: Recommend scheduling next health check",
        if include_recommendations {
            "## Recommendations\n\
            ### Immediate Actions\n\
            [Actions required within 24 hours]\n\n\
            ### Short-term Improvements\n\
            [Optimizations for next 1-2 weeks]\n\n\
            ### Long-term Planning\n\
            [Strategic improvements for system growth]"
        } else {
            ""
        }
    )
}
