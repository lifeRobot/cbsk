/// datetime serialize
pub trait DateTimeSerialize {
    /// get year
    fn get_year(&self) -> i32;

    /// get month
    fn get_month(&self) -> u8;

    /// get day
    fn get_day(&self) -> u8;

    /// get week day
    fn get_week_day(&self) -> u8;

    /// get hour
    fn get_hour(&self) -> u8;

    /// get minute
    fn get_minute(&self) -> u8;

    /// get second
    fn get_second(&self) -> u8;

    /// get nanosecond
    fn get_nano(&self) -> u32;

    /// get yyyy-mm-ddThh:mm:ss.nnnnnnnnn format datetime
    fn yyyy_mm_dd_t_hh_mm_ss_n(&self) -> String {
        format!("{}.{:09}", self.yyyy_mm_dd_t_hh_mm_ss(), self.get_nano())
    }

    /// get yyyy-mm-ddThh:mm:ss format datetime
    fn yyyy_mm_dd_t_hh_mm_ss(&self) -> String {
        format!("{}T{}", self.yyyy_mm_dd(), self.hh_mm_ss())
    }

    /// get yyyy-mm-dd hh:mm:ss.nnnnnnnnn format datetime
    fn yyyy_mm_dd_hh_mm_ss_n(&self) -> String {
        format!("{}.{:09}", self.yyyy_mm_dd_hh_mm_ss(), self.get_nano())
    }

    /// get yyyy-mm-dd hh:mm:ss format datetime
    fn yyyy_mm_dd_hh_mm_ss(&self) -> String {
        format!("{} {}", self.yyyy_mm_dd(), self.hh_mm_ss())
    }

    /// get yyyymmddhhmmss.nnnnnnnnn format datetime
    fn yyyymmddhhmmss_n(&self) -> String {
        format!("{}.{:09}", self.yyyymmddhhmmss(), self.get_nano())
    }

    /// get yyyymmddhhmmss format datetime
    fn yyyymmddhhmmss(&self) -> String {
        format!("{}{:06}", self.yyyymmdd(), self.hhmmss())
    }

    /// get yyyy-mm-dd format date
    fn yyyy_mm_dd(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.get_year(), self.get_month(), self.get_day())
    }

    /// get hh:mm:ss format time
    fn hh_mm_ss(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.get_hour(), self.get_minute(), self.get_second())
    }

    /// get yyyymmdd format date
    fn yyyymmdd(&self) -> i32 {
        self.yyyymm() * 100 + i32::from(self.get_day())
    }

    /// get yyyymmww format date<br />
    /// ww is week, such as monday is 01, sunday is 07
    fn yyyymmww(&self) -> i32 {
        self.yyyymm() * 100 + i32::from(self.get_week_day())
    }

    /// get yyyymm format date
    fn yyyymm(&self) -> i32 {
        self.get_year() * 100 + i32::from(self.get_month())
    }

    /// get yyyymmss format time
    fn hhmmss(&self) -> u32 {
        u32::from(self.hhmm()) * 100 + u32::from(self.get_second())
    }

    /// get hhmm format time
    fn hhmm(&self) -> u16 {
        u16::from(self.get_hour()) * 100 + u16::from(self.get_minute())
    }
}

/// support datetime default serialize
#[cfg(feature = "fastdate")]
impl DateTimeSerialize for fastdate::DateTime {
    fn get_year(&self) -> i32 {
        self.year()
    }
    fn get_month(&self) -> u8 {
        self.mon()
    }
    fn get_day(&self) -> u8 {
        self.day()
    }
    fn get_week_day(&self) -> u8 {
        self.week_day()
    }
    fn get_hour(&self) -> u8 {
        self.hour()
    }
    fn get_minute(&self) -> u8 {
        self.minute()
    }
    fn get_second(&self) -> u8 {
        self.sec()
    }
    fn get_nano(&self) -> u32 {
        self.nano()
    }
}
