-- =========================================================================
-- 잠실점 에비뉴엘 시설물 및 배관 통합 관제 시스템 (Avenuel Building FMS)
-- Supabase PostgreSQL 데이터베이스 스키마 및 초기 데이터 마이그레이션
-- =========================================================================

-- 1. updated_at 자동 갱신 트리거 함수
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 2. 매장 테이블 (shops)
CREATE TABLE IF NOT EXISTS public.shops (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    floor TEXT NOT NULL DEFAULT '2F',
    description TEXT,
    pipe_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TRIGGER update_shops_updated_at
BEFORE UPDATE ON public.shops
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 3. 배관 및 전기 라인 테이블 (pipes)
CREATE TABLE IF NOT EXISTS public.pipes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    shop TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('PLUMBING', 'FIRE', 'ELEC', 'HVAC')),
    spec TEXT NOT NULL,
    length TEXT NOT NULL,
    points TEXT NOT NULL,
    color TEXT NOT NULL,
    floor TEXT NOT NULL DEFAULT '2F',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TRIGGER update_pipes_updated_at
BEFORE UPDATE ON public.pipes
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 4. 시설물 핀 테이블 (facilities)
CREATE TABLE IF NOT EXISTS public.facilities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    shop TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('PLUMBING', 'FIRE', 'ELEC', 'HVAC')),
    floor TEXT NOT NULL DEFAULT '2F',
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL DEFAULT '정상',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TRIGGER update_facilities_updated_at
BEFORE UPDATE ON public.facilities
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 5. 직원 및 PIN 인증 테이블 (staff)
CREATE TABLE IF NOT EXISTS public.staff (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    pin TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'viewer')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TRIGGER update_staff_updated_at
BEFORE UPDATE ON public.staff
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 6. 구글 드라이브 / 시트 백업 이력 관리 테이블 (backup_logs)
CREATE TABLE IF NOT EXISTS public.backup_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_type TEXT NOT NULL CHECK (backup_type IN ('FULL', 'INCREMENTAL')),
    target TEXT NOT NULL DEFAULT 'GOOGLE_DRIVE',
    file_name TEXT NOT NULL,
    file_id TEXT,
    records_count INTEGER NOT NULL DEFAULT 0,
    since_timestamp TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'SUCCESS',
    details JSONB
);

-- 7. RLS (Row Level Security) 활성화 및 전체 읽기/쓰기 정책 허용 (FMS 대시보드 연동용)
ALTER TABLE public.shops ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.pipes ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.facilities ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.staff ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.backup_logs ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Allow public read access on shops" ON public.shops FOR SELECT USING (true);
CREATE POLICY "Allow public insert/update/delete on shops" ON public.shops FOR ALL USING (true);

CREATE POLICY "Allow public read access on pipes" ON public.pipes FOR SELECT USING (true);
CREATE POLICY "Allow public insert/update/delete on pipes" ON public.pipes FOR ALL USING (true);

CREATE POLICY "Allow public read access on facilities" ON public.facilities FOR SELECT USING (true);
CREATE POLICY "Allow public insert/update/delete on facilities" ON public.facilities FOR ALL USING (true);

CREATE POLICY "Allow public read access on staff" ON public.staff FOR SELECT USING (true);
CREATE POLICY "Allow public insert/update/delete on staff" ON public.staff FOR ALL USING (true);

CREATE POLICY "Allow public read access on backup_logs" ON public.backup_logs FOR SELECT USING (true);
CREATE POLICY "Allow public insert on backup_logs" ON public.backup_logs FOR ALL USING (true);

-- 8. 초기 잠실점 에비뉴엘 2F 데이터 시드
INSERT INTO public.shops (id, name, code, floor, description, pipe_count) VALUES
('S-01', '샤넬', 'CHANEL', '2F', '급수(50A), 전력(75kW), 소방SP 인입', 3),
('S-02', '에르메스', 'HERMES', '2F', '급수(65A), 간선전력(50kW) 인입', 2),
('S-03', '루이비통', 'LOUIS VUITTON', '2F', '급수(50A), 특고압(80kW) 인입', 2),
('S-04', '디올', 'DIOR', '2F', '급수(40A), 조명간선(45kW) 인입', 2),
('S-05', '구찌', 'GUCCI', '2F', '급수(50A), 전력선(60kW) 인입', 2),
('S-06', '롤렉스', 'ROLEX', '2F', '공조덕트, 분전반 인입', 1),
('S-07', '까르띠에', 'CARTIER', '2F', '급수, 전력 배선 연결 대기', 0),
('S-08', '티파니', 'TIFFANY & CO.', '2F', '조명선로 인입 완료', 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO public.pipes (id, name, shop, category, spec, length, points, color) VALUES
('P-201', '샤넬 인입 전용 급수관 50A', '샤넬', 'PLUMBING', '50A 스테인리스', '18.4m', '470,300 115,300 115,270', '#0284c7'),
('P-202', '샤넬 메인 3상 수전간선 75kW', '샤넬', 'ELEC', 'F-CV 35sq', '21.0m', '460,290 125,290 125,270', '#d97706'),
('P-203', '샤넬 습식 스프링클러 배관', '샤넬', 'FIRE', '80A 배관', '15.6m', '470,310 105,310 105,270', '#dc2626'),
('P-204', '에르메스 급수/오수 메인 배관', '에르메스', 'PLUMBING', '65A 스테인리스', '23.8m', '470,300 285,300 285,270', '#0284c7'),
('P-205', '에르메스 전력 간선 분전선', '에르메스', 'ELEC', 'F-CV 25sq', '19.2m', '460,290 295,290 295,270', '#d97706'),
('P-206', '루이비통 전용 급수 배관 50A', '루이비통', 'PLUMBING', '50A 스테인리스', '24.5m', '470,300 655,300 655,270', '#0284c7'),
('P-207', '루이비통 대용량 수전설비 80kW', '루이비통', 'ELEC', 'F-CV 50sq', '26.8m', '460,290 645,290 645,270', '#d97706'),
('P-208', '디올 인입 급수관 40A', '디올', 'PLUMBING', '40A 스테인리스', '31.2m', '470,300 825,300 825,270', '#0284c7'),
('P-209', '디올 조명 및 전력 간선 45kW', '디올', 'ELEC', 'F-CV 25sq', '33.0m', '460,290 835,290 835,270', '#d97706'),
('P-210', '구찌 인입 급수 라인 50A', '구찌', 'PLUMBING', '50A 배관', '18.1m', '470,300 115,300 115,330', '#0284c7'),
('P-211', '구찌 메인 전력선 60kW', '구찌', 'ELEC', 'F-CV 35sq', '19.7m', '460,290 125,290 125,330', '#d97706'),
('P-212', '롤렉스 공조 급기 덕트 300x200', '롤렉스', 'HVAC', '아연도강판 덕트', '22.4m', '470,320 285,320 285,330', '#059669')
ON CONFLICT (id) DO NOTHING;

INSERT INTO public.facilities (id, name, shop, category, floor, x, y, status) VALUES
('F-101', '샤넬 옥내소화전 2-1', '샤넬', 'FIRE', '2F', 50.0, 50.0, '정상'),
('F-102', '샤넬 조명 분전반 LP-2A', '샤넬', 'ELEC', '2F', 170.0, 50.0, '정상'),
('F-103', '에르메스 소방 밸브 스테이션', '에르메스', 'FIRE', '2F', 220.0, 50.0, '정상'),
('F-104', '루이비통 메인 배전반 MP-2B', '루이비통', 'ELEC', '2F', 710.0, 50.0, '정상'),
('F-105', '디올 FCU 공조기 2대', '디올', 'HVAC', '2F', 880.0, 50.0, '정상')
ON CONFLICT (id) DO NOTHING;

INSERT INTO public.staff (id, name, pin, role) VALUES
('ST-01', '김시설 관리자', '1234', 'admin'),
('ST-02', '이순찰 직원', '5678', 'viewer')
ON CONFLICT (id) DO NOTHING;

-- 8. Supabase 실시간 동기화(Realtime) 복제 활성화
ALTER PUBLICATION supabase_realtime ADD TABLE public.shops;
ALTER PUBLICATION supabase_realtime ADD TABLE public.pipes;
ALTER PUBLICATION supabase_realtime ADD TABLE public.facilities;
