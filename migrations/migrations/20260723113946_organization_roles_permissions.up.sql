-- Custom organization roles and permission matrix for migration parity with RocketChat.
CREATE TABLE organization_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    description TEXT,
    UNIQUE (organization_id, key)
);

CREATE TABLE organization_role_permissions (
    role_id UUID NOT NULL REFERENCES organization_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_organization_roles_org ON organization_roles (organization_id);
CREATE INDEX idx_permissions_org ON permissions (organization_id);
CREATE INDEX idx_role_permissions_role ON organization_role_permissions (role_id);
