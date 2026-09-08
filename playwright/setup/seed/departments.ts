// The three departments the members are spread across.
//
// `departments` is keyed by a generated UUID and unique on `name`, and
// membership is `user_profile_ext.department` holding the NAME — so the rows
// are upserted on name and never deleted by the reset: a developer may have
// hand-assigned a real user to Engineering, and the seed owns only the
// principals' assignments (written in principals.ts), not the department.
import type { Client } from 'pg';
import { E2E } from './principals';

const DEPARTMENTS: { name: string; description: string }[] = [
  { name: E2E.departments.engineering, description: 'Builds and ships the product.' },
  { name: E2E.departments.product, description: 'Decides what gets built.' },
  { name: E2E.departments.support, description: 'Keeps customers running.' },
];

export async function seedDepartments(db: Client) {
  for (const d of DEPARTMENTS) {
    await db.query(
      `INSERT INTO departments (name, description) VALUES ($1, $2)
       ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description, updated_at = NOW()`,
      [d.name, d.description],
    );
  }
}

/** The id the console addresses a seeded department by. */
export async function departmentId(db: Client, name: string): Promise<string> {
  const { rows } = await db.query<{ id: string }>('SELECT id FROM departments WHERE name = $1', [
    name,
  ]);
  const id = rows[0]?.id;
  if (!id) throw new Error(`department ${name} is not seeded`);
  return id;
}
