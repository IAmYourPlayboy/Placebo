use crate::db::camera::{self, Camera};
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn seed_cameras(pool: &SqlitePool) -> Result<usize> {
    let existing = camera::get_count(pool).await?;
    if existing > 0 {
        return Ok(0);
    }

    let cameras = seed_data();
    let count = camera::insert_batch(pool, &cameras).await?;
    Ok(count)
}

fn cam(
    id: &str, name: &str, slug: &str,
    lat: f64, lng: f64,
    country: &str, country_code: &str, city: &str,
    category: &str, stream_type: &str,
    timezone: &str, description: &str,
) -> Camera {
    let mut c = camera::new_camera(
        id, name, slug, lat, lng,
        &format!("rtsp://cam.placebo.tv/{}/live", slug),
    );
    c.country = Some(country.to_string());
    c.country_code = Some(country_code.to_string());
    c.city = Some(city.to_string());
    c.category = category.to_string();
    c.stream_type = Some(stream_type.to_string());
    c.timezone = Some(timezone.to_string());
    c.description_en = Some(description.to_string());
    c.codec = Some("h264".to_string());
    c.resolution_w = Some(1920);
    c.resolution_h = Some(1080);
    c
}

fn seed_data() -> Vec<Camera> {
    vec![
        // ─── Tokyo (7) ─────────────────────────────────────────────────
        cam("tok-001", "渋谷スクランブル交差点", "shibuya-crossing",
            35.6595, 139.7004, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Shibuya Crossing – the world's busiest pedestrian intersection"),
        cam("tok-002", "東京タワー", "tokyo-tower",
            35.6586, 139.7454, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Tokyo Tower panoramic view"),
        cam("tok-003", "新宿歌舞伎町", "shinjuku-kabukicho",
            35.6938, 139.7034, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Shinjuku Kabukicho entertainment district"),
        cam("tok-004", "秋葉原電気街", "akihabara-electric-town",
            35.6984, 139.7731, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Akihabara Electric Town main street"),
        cam("tok-005", "浅草寺雷門", "asakusa-kaminarimon",
            35.7115, 139.7966, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Senso-ji Temple Kaminarimon Gate"),
        cam("tok-006", "お台場レインボーブリッジ", "odaiba-rainbow-bridge",
            35.6308, 139.7753, "Japan", "JP", "Tokyo",
            "city", "rtsp", "Asia/Tokyo",
            "Rainbow Bridge viewed from Odaiba"),
        cam("tok-007", "首都高速", "shuto-expressway",
            35.6762, 139.6503, "Japan", "JP", "Tokyo",
            "traffic", "rtsp", "Asia/Tokyo",
            "Shuto Expressway traffic camera"),

        // ─── Moscow (7) ────────────────────────────────────────────────
        cam("msk-001", "Красная площадь", "red-square",
            55.7539, 37.6208, "Russia", "RU", "Moscow",
            "city", "rtsp", "Europe/Moscow",
            "Red Square – heart of Moscow"),
        cam("msk-002", "МКАД Кольцевая", "mkad-ring-road",
            55.7700, 37.8420, "Russia", "RU", "Moscow",
            "traffic", "rtsp", "Europe/Moscow",
            "MKAD ring road traffic"),
        cam("msk-003", "Москва-река набережная", "moscow-river-embankment",
            55.7470, 37.6050, "Russia", "RU", "Moscow",
            "nature", "rtsp", "Europe/Moscow",
            "Moscow River embankment near Kremlin"),
        cam("msk-004", "Тверская улица", "tverskaya-street",
            55.7650, 37.6060, "Russia", "RU", "Moscow",
            "city", "rtsp", "Europe/Moscow",
            "Tverskaya Street – main avenue"),
        cam("msk-005", "Парк Горького", "gorky-park",
            55.7312, 37.6030, "Russia", "RU", "Moscow",
            "nature", "rtsp", "Europe/Moscow",
            "Gorky Park central entrance"),
        cam("msk-006", "Москва-Сити", "moscow-city",
            55.7494, 37.5367, "Russia", "RU", "Moscow",
            "city", "rtsp", "Europe/Moscow",
            "Moscow International Business Center skyline"),
        cam("msk-007", "Воробьёвы горы", "sparrow-hills",
            55.7105, 37.5425, "Russia", "RU", "Moscow",
            "nature", "rtsp", "Europe/Moscow",
            "Sparrow Hills panoramic view of Moscow"),

        // ─── New York (6) ──────────────────────────────────────────────
        cam("nyc-001", "Times Square", "times-square",
            40.7580, -73.9855, "United States", "US", "New York",
            "city", "rtsp", "America/New_York",
            "Times Square – the crossroads of the world"),
        cam("nyc-002", "Brooklyn Bridge", "brooklyn-bridge",
            40.7061, -73.9969, "United States", "US", "New York",
            "city", "rtsp", "America/New_York",
            "Brooklyn Bridge pedestrian walkway"),
        cam("nyc-003", "Central Park", "central-park",
            40.7829, -73.9654, "United States", "US", "New York",
            "nature", "rtsp", "America/New_York",
            "Central Park Bethesda Fountain area"),
        cam("nyc-004", "Empire State Building", "empire-state-building",
            40.7484, -73.9857, "United States", "US", "New York",
            "city", "rtsp", "America/New_York",
            "Empire State Building observation deck"),
        cam("nyc-005", "5th Avenue Traffic", "5th-avenue-traffic",
            40.7549, -73.9840, "United States", "US", "New York",
            "traffic", "rtsp", "America/New_York",
            "5th Avenue traffic near Rockefeller Center"),
        cam("nyc-006", "Statue of Liberty", "statue-of-liberty",
            40.6892, -74.0445, "United States", "US", "New York",
            "city", "rtsp", "America/New_York",
            "Statue of Liberty from Liberty Island"),

        // ─── Mumbai (6) ────────────────────────────────────────────────
        cam("mum-001", "Marine Drive", "marine-drive",
            18.9438, 72.8234, "India", "IN", "Mumbai",
            "city", "rtsp", "Asia/Kolkata",
            "Marine Drive – Queen's Necklace"),
        cam("mum-002", "Gateway of India", "gateway-of-india",
            18.9220, 72.8347, "India", "IN", "Mumbai",
            "city", "rtsp", "Asia/Kolkata",
            "Gateway of India monument"),
        cam("mum-003", "Bandra-Worli Sea Link", "bandra-worli-sealink",
            19.0380, 72.8162, "India", "IN", "Mumbai",
            "traffic", "rtsp", "Asia/Kolkata",
            "Bandra-Worli Sea Link bridge traffic"),
        cam("mum-004", "Juhu Beach", "juhu-beach",
            19.0988, 72.8267, "India", "IN", "Mumbai",
            "nature", "rtsp", "Asia/Kolkata",
            "Juhu Beach sunset view"),
        cam("mum-005", "CST Station", "cst-station",
            18.9398, 72.8355, "India", "IN", "Mumbai",
            "city", "rtsp", "Asia/Kolkata",
            "Chhatrapati Shivaji Terminus – UNESCO heritage"),
        cam("mum-006", "Haji Ali Dargah", "haji-ali-dargah",
            18.9827, 72.8089, "India", "IN", "Mumbai",
            "city", "rtsp", "Asia/Kolkata",
            "Haji Ali Dargah mosque on islet"),

        // ─── Helsinki (6) ──────────────────────────────────────────────
        cam("hel-001", "Сенатская площадь", "senate-square",
            60.1695, 24.9527, "Finland", "FI", "Helsinki",
            "city", "rtsp", "Europe/Helsinki",
            "Senate Square with Helsinki Cathedral"),
        cam("hel-002", "South Harbor", "south-harbor",
            60.1670, 24.9560, "Finland", "FI", "Helsinki",
            "harbor", "rtsp", "Europe/Helsinki",
            "South Harbor market square and ferries"),
        cam("hel-003", "Esplanadi Park", "esplanadi-park",
            60.1674, 24.9451, "Finland", "FI", "Helsinki",
            "nature", "rtsp", "Europe/Helsinki",
            "Esplanadi Park promenade"),
        cam("hel-004", "Suomenlinna", "suomenlinna",
            60.1454, 24.9881, "Finland", "FI", "Helsinki",
            "nature", "rtsp", "Europe/Helsinki",
            "Suomenlinna sea fortress – UNESCO site"),
        cam("hel-005", "Mannerheimintie", "mannerheimintie",
            60.1718, 24.9414, "Finland", "FI", "Helsinki",
            "traffic", "rtsp", "Europe/Helsinki",
            "Mannerheimintie main avenue traffic"),
        cam("hel-006", "Temppeliaukio Church", "temppeliaukio-church",
            60.1729, 24.9252, "Finland", "FI", "Helsinki",
            "city", "rtsp", "Europe/Helsinki",
            "Rock Church exterior view"),

        // ─── London (6) ────────────────────────────────────────────────
        cam("lon-001", "Tower Bridge", "tower-bridge",
            51.5055, -0.0754, "United Kingdom", "GB", "London",
            "city", "rtsp", "Europe/London",
            "Tower Bridge over the Thames"),
        cam("lon-002", "Trafalgar Square", "trafalgar-square",
            51.5080, -0.1281, "United Kingdom", "GB", "London",
            "city", "rtsp", "Europe/London",
            "Trafalgar Square with Nelson's Column"),
        cam("lon-003", "Westminster Bridge", "westminster-bridge",
            51.5007, -0.1218, "United Kingdom", "GB", "London",
            "city", "rtsp", "Europe/London",
            "Westminster Bridge and Big Ben view"),
        cam("lon-004", "Piccadilly Circus", "piccadilly-circus",
            51.5101, -0.1340, "United Kingdom", "GB", "London",
            "city", "rtsp", "Europe/London",
            "Piccadilly Circus neon lights"),
        cam("lon-005", "Thames River", "thames-river",
            51.5074, -0.0985, "United Kingdom", "GB", "London",
            "nature", "rtsp", "Europe/London",
            "Thames River south bank"),
        cam("lon-006", "Oxford Street", "oxford-street",
            51.5152, -0.1418, "United Kingdom", "GB", "London",
            "traffic", "rtsp", "Europe/London",
            "Oxford Street shopping traffic"),

        // ─── Paris (6) ─────────────────────────────────────────────────
        cam("par-001", "Tour Eiffel", "eiffel-tower",
            48.8584, 2.2945, "France", "FR", "Paris",
            "city", "rtsp", "Europe/Paris",
            "Eiffel Tower from Trocadéro"),
        cam("par-002", "Champs-Élysées", "champs-elysees",
            48.8698, 2.3076, "France", "FR", "Paris",
            "city", "rtsp", "Europe/Paris",
            "Champs-Élysées avenue towards Arc de Triomphe"),
        cam("par-003", "Notre-Dame", "notre-dame",
            48.8530, 2.3499, "France", "FR", "Paris",
            "city", "rtsp", "Europe/Paris",
            "Notre-Dame Cathedral reconstruction"),
        cam("par-004", "Seine River", "seine-river",
            48.8566, 2.3522, "France", "FR", "Paris",
            "nature", "rtsp", "Europe/Paris",
            "Seine River near Île de la Cité"),
        cam("par-005", "Montmartre", "montmartre",
            48.8867, 2.3431, "France", "FR", "Paris",
            "city", "rtsp", "Europe/Paris",
            "Montmartre Sacré-Cœur view"),
        cam("par-006", "Place de la Concorde", "place-de-la-concorde",
            48.8656, 2.3212, "France", "FR", "Paris",
            "traffic", "rtsp", "Europe/Paris",
            "Place de la Concorde roundabout"),

        // ─── Dubai (6) ─────────────────────────────────────────────────
        cam("dxb-001", "Burj Khalifa", "burj-khalifa",
            25.1972, 55.2744, "United Arab Emirates", "AE", "Dubai",
            "city", "rtsp", "Asia/Dubai",
            "Burj Khalifa – tallest building in the world"),
        cam("dxb-002", "Dubai Marina", "dubai-marina",
            25.0805, 55.1403, "United Arab Emirates", "AE", "Dubai",
            "city", "rtsp", "Asia/Dubai",
            "Dubai Marina skyline and yachts"),
        cam("dxb-003", "Palm Jumeirah", "palm-jumeirah",
            25.1124, 55.1390, "United Arab Emirates", "AE", "Dubai",
            "city", "rtsp", "Asia/Dubai",
            "Palm Jumeirah aerial view"),
        cam("dxb-004", "Dubai Creek", "dubai-creek",
            25.2644, 55.2975, "United Arab Emirates", "AE", "Dubai",
            "nature", "rtsp", "Asia/Dubai",
            "Dubai Creek traditional dhow boats"),
        cam("dxb-005", "Sheikh Zayed Road", "sheikh-zayed-road",
            25.2048, 55.2708, "United Arab Emirates", "AE", "Dubai",
            "traffic", "rtsp", "Asia/Dubai",
            "Sheikh Zayed Road traffic – main highway"),
        cam("dxb-006", "Jumeirah Beach", "jumeirah-beach",
            25.2048, 55.2340, "United Arab Emirates", "AE", "Dubai",
            "nature", "rtsp", "Asia/Dubai",
            "Jumeirah Beach with Burj Al Arab backdrop"),
    ]
}
