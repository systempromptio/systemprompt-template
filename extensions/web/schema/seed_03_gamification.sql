-- Seed realistic gamification data for demonstration
-- Uses existing user IDs from the users table
-- Only inserts if employee_ranks is empty (idempotent)

DO $$
DECLARE
    user_ids TEXT[];
    uid TEXT;
    u_idx INT;
    user_count INT;
    day_offset INT;
    ev_count INT;
    row_count INT;

    -- Rank thresholds: level => (min_xp, rank_name)
    -- 1: Spark=0, 2: Prompt Apprentice=50, 3: Token Tinkerer=150,
    -- 4: Context Crafter=400, 5: Neural Navigator=800, 6: Model Whisperer=1500,
    -- 7: Pipeline Architect=3000, 8: Singularity Sage=5000, 9: Emergent Mind=8000,
    -- 10: Superintelligence=12000

    rank_names TEXT[] := ARRAY[
        'Spark', 'Prompt Apprentice', 'Token Tinkerer', 'Context Crafter',
        'Neural Navigator', 'Model Whisperer', 'Pipeline Architect',
        'Singularity Sage', 'Emergent Mind', 'Superintelligence'
    ];
    rank_thresholds INT[] := ARRAY[0, 50, 150, 400, 800, 1500, 3000, 5000, 8000, 12000];

    -- Per-user target XP values (spread across ranks, assigned round-robin)
    target_xps INT[] := ARRAY[
        13500, 9200, 5800, 3400, 1800, 950, 520, 200, 75, 20,
        12200, 8500, 4200, 2100, 1200, 600, 300, 120, 40, 10
    ];
    target_xp INT;
    r_level INT;
    r_name TEXT;

    -- Achievement pools by tier
    first_achievements TEXT[] := ARRAY['first_spark', 'first_tool', 'first_agent', 'first_mcp', 'first_custom', 'first_plugin'];
    usage_achievements TEXT[] := ARRAY['prompts_10', 'prompts_50', 'prompts_100', 'prompts_250', 'prompts_500', 'prompts_1000'];
    skill_achievements TEXT[] := ARRAY['skills_5', 'skills_15', 'skills_30'];
    plugin_achievements TEXT[] := ARRAY['plugins_3', 'plugins_all'];
    social_achievements TEXT[] := ARRAY['master_skill', 'dept_champion', 'org_top10', 'custom_3', 'share_skill', 'team_50'];
    streak_achievements TEXT[] := ARRAY['streak_3', 'streak_7', 'streak_14', 'streak_30', 'streak_60'];

    ach TEXT;
    ach_ts TIMESTAMPTZ;
    streak INT;
    longest INT;
    skills_count INT;
    plugins_count INT;
    events_total BIGINT;
    last_date DATE;
BEGIN
    SELECT COUNT(*) INTO row_count FROM employee_ranks;
    IF row_count > 0 THEN
        RAISE NOTICE 'employee_ranks already has data, skipping gamification seed';
        RETURN;
    END IF;

    SELECT ARRAY_AGG(id) INTO user_ids FROM users WHERE department != '' AND department IS NOT NULL;

    IF user_ids IS NULL OR array_length(user_ids, 1) IS NULL THEN
        RAISE NOTICE 'No users with departments found, skipping gamification seed';
        RETURN;
    END IF;

    user_count := array_length(user_ids, 1);

    -- Iterate over each user and assign gamification data
    FOR u_idx IN 1..user_count LOOP
        uid := user_ids[u_idx];
        target_xp := target_xps[1 + ((u_idx - 1) % array_length(target_xps, 1))];

        -- Determine rank level from XP
        r_level := 1;
        r_name := rank_names[1];
        FOR i IN REVERSE 10..1 LOOP
            IF target_xp >= rank_thresholds[i] THEN
                r_level := i;
                r_name := rank_names[i];
                EXIT;
            END IF;
        END LOOP;

        -- Determine streak based on rank (higher rank = longer streaks)
        streak := GREATEST(0, r_level - 1 + floor(random() * 4)::INT);
        longest := streak + floor(random() * (r_level * 3))::INT;
        IF longest < streak THEN longest := streak; END IF;

        -- Skill and plugin counts scale with rank
        skills_count := LEAST(20, r_level + floor(random() * r_level * 2)::INT);
        plugins_count := LEAST(8, 1 + floor(random() * LEAST(8, r_level))::INT);

        -- Events total correlates with XP
        events_total := GREATEST(1, (target_xp * (80 + floor(random() * 40)::INT)) / 100);

        -- Last active date: higher ranked users more recently active
        IF r_level >= 7 THEN
            last_date := CURRENT_DATE;
        ELSIF r_level >= 4 THEN
            last_date := CURRENT_DATE - floor(random() * 3)::INT;
        ELSE
            last_date := CURRENT_DATE - floor(random() * 10)::INT;
        END IF;

        ---------------------------------------------------------------
        -- 1. employee_ranks
        ---------------------------------------------------------------
        INSERT INTO employee_ranks (
            user_id, total_xp, rank_level, rank_name, events_count,
            unique_skills_count, unique_plugins_count,
            current_streak, longest_streak, last_active_date, updated_at
        ) VALUES (
            uid, target_xp, r_level, r_name, events_total,
            skills_count, plugins_count,
            streak, longest, last_date, NOW()
        );

        ---------------------------------------------------------------
        -- 2. employee_daily_usage (past 30 days)
        ---------------------------------------------------------------
        FOR day_offset IN 0..29 LOOP
            -- Higher-ranked users have higher daily probability and counts
            IF random() < (0.3 + (r_level::FLOAT / 15.0)) THEN
                ev_count := GREATEST(1, floor(random() * r_level * 5)::INT + 1);
                INSERT INTO employee_daily_usage (user_id, usage_date, event_count)
                VALUES (uid, CURRENT_DATE - day_offset, ev_count);
            END IF;
        END LOOP;

        ---------------------------------------------------------------
        -- 3. employee_achievements
        ---------------------------------------------------------------
        -- Everyone gets first_spark
        INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
        VALUES (gen_random_uuid()::TEXT, uid, 'first_spark',
                NOW() - ((30 + floor(random() * 60))::TEXT || ' days')::INTERVAL);

        -- First-tier achievements: scale with rank
        FOREACH ach IN ARRAY first_achievements[2:LEAST(array_length(first_achievements, 1), r_level + 1)] LOOP
            ach_ts := NOW() - ((floor(random() * 45) + 5)::TEXT || ' days')::INTERVAL;
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, ach, ach_ts);
        END LOOP;

        -- Usage achievements: unlocked based on events/xp
        IF target_xp >= 50 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_10',
                    NOW() - ((20 + floor(random() * 30))::TEXT || ' days')::INTERVAL);
        END IF;
        IF target_xp >= 200 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_50',
                    NOW() - ((15 + floor(random() * 20))::TEXT || ' days')::INTERVAL);
        END IF;
        IF target_xp >= 800 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_100',
                    NOW() - ((10 + floor(random() * 15))::TEXT || ' days')::INTERVAL);
        END IF;
        IF target_xp >= 1500 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_250',
                    NOW() - ((5 + floor(random() * 12))::TEXT || ' days')::INTERVAL);
        END IF;
        IF target_xp >= 3000 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_500',
                    NOW() - ((2 + floor(random() * 8))::TEXT || ' days')::INTERVAL);
        END IF;
        IF target_xp >= 8000 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'prompts_1000',
                    NOW() - ((1 + floor(random() * 5))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Skill achievements
        IF skills_count >= 5 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'skills_5',
                    NOW() - ((10 + floor(random() * 20))::TEXT || ' days')::INTERVAL);
        END IF;
        IF skills_count >= 15 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'skills_15',
                    NOW() - ((5 + floor(random() * 10))::TEXT || ' days')::INTERVAL);
        END IF;
        IF skills_count >= 20 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'skills_30',
                    NOW() - ((1 + floor(random() * 5))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Plugin achievements
        IF plugins_count >= 3 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'plugins_3',
                    NOW() - ((8 + floor(random() * 15))::TEXT || ' days')::INTERVAL);
        END IF;
        IF plugins_count >= 8 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'plugins_all',
                    NOW() - ((1 + floor(random() * 5))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Streak achievements
        IF longest >= 3 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'streak_3',
                    NOW() - ((20 + floor(random() * 20))::TEXT || ' days')::INTERVAL);
        END IF;
        IF longest >= 7 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'streak_7',
                    NOW() - ((12 + floor(random() * 15))::TEXT || ' days')::INTERVAL);
        END IF;
        IF longest >= 14 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'streak_14',
                    NOW() - ((6 + floor(random() * 10))::TEXT || ' days')::INTERVAL);
        END IF;
        IF longest >= 30 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'streak_30',
                    NOW() - ((2 + floor(random() * 5))::TEXT || ' days')::INTERVAL);
        END IF;
        IF longest >= 60 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'streak_60',
                    NOW() - ((1 + floor(random() * 3))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Social/special achievements for high-rank users
        IF r_level >= 5 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'share_skill',
                    NOW() - ((5 + floor(random() * 10))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 6 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'custom_3',
                    NOW() - ((3 + floor(random() * 8))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 7 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'master_skill',
                    NOW() - ((2 + floor(random() * 5))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 8 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'dept_champion',
                    NOW() - ((1 + floor(random() * 4))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 9 THEN
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'org_top10',
                    NOW() - ((1 + floor(random() * 3))::TEXT || ' days')::INTERVAL);
            INSERT INTO employee_achievements (id, user_id, achievement_id, unlocked_at)
            VALUES (gen_random_uuid()::TEXT, uid, 'team_50',
                    NOW() - ((1 + floor(random() * 3))::TEXT || ' days')::INTERVAL);
        END IF;

        ---------------------------------------------------------------
        -- 4. employee_xp_ledger (bonus entries)
        ---------------------------------------------------------------
        -- Streak bonus XP for users with streaks
        IF streak >= 7 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 25, 'streak_bonus',
                    'streak_7',
                    NOW() - ((floor(random() * 14))::TEXT || ' days')::INTERVAL);
        END IF;
        IF streak >= 14 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 50, 'streak_bonus',
                    'streak_14',
                    NOW() - ((floor(random() * 7))::TEXT || ' days')::INTERVAL);
        END IF;
        IF streak >= 30 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 100, 'streak_bonus',
                    'streak_30',
                    NOW() - ((floor(random() * 3))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Achievement unlock XP bonuses for high-rank users
        IF r_level >= 4 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 50, 'achievement_bonus',
                    'prompts_100',
                    NOW() - ((10 + floor(random() * 15))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 6 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 100, 'achievement_bonus',
                    'master_skill',
                    NOW() - ((3 + floor(random() * 8))::TEXT || ' days')::INTERVAL);
        END IF;
        IF r_level >= 8 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid, 200, 'achievement_bonus',
                    'dept_champion',
                    NOW() - ((1 + floor(random() * 4))::TEXT || ' days')::INTERVAL);
        END IF;

        -- Random weekly bonus for some users
        IF random() < 0.4 AND r_level >= 3 THEN
            INSERT INTO employee_xp_ledger (id, user_id, xp_amount, source, source_id, created_at)
            VALUES (gen_random_uuid()::TEXT, uid,
                    10 + floor(random() * 40)::INT,
                    'weekly_bonus', NULL,
                    NOW() - ((floor(random() * 7))::TEXT || ' days')::INTERVAL);
        END IF;

    END LOOP;

    RAISE NOTICE 'Seeded gamification data for % users', user_count;
END $$;
