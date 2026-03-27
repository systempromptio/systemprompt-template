mod helpers_format;
mod helpers_logic;

use helpers_format::{
    ConcatHelper, CssVersionHelper, DefaultHelper, FormatDateHelper, InitialsHelper, JsonHelper,
    RelativeTimeHelper, ToLowerCaseHelper, TruncateHelper,
};
use helpers_logic::{AddHelper, EqHelper, GtHelper, NotHelper, SubHelper};

pub fn register_helpers(hbs: &mut handlebars::Handlebars<'static>) {
    hbs.register_helper("formatDate", Box::new(FormatDateHelper));
    hbs.register_helper("relativeTime", Box::new(RelativeTimeHelper));
    hbs.register_helper("initials", Box::new(InitialsHelper));
    hbs.register_helper("truncate", Box::new(TruncateHelper));
    hbs.register_helper("json", Box::new(JsonHelper));
    hbs.register_helper("concat", Box::new(ConcatHelper));
    hbs.register_helper("toLowerCase", Box::new(ToLowerCaseHelper));
    hbs.register_helper("default", Box::new(DefaultHelper));
    hbs.register_helper("css_version", Box::new(CssVersionHelper));
    hbs.register_helper("eq", Box::new(EqHelper));
    hbs.register_helper("gt", Box::new(GtHelper));
    hbs.register_helper("not", Box::new(NotHelper));
    hbs.register_helper("add", Box::new(AddHelper));
    hbs.register_helper("sub", Box::new(SubHelper));
}
