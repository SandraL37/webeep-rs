#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteInfo {
    sitename: String,
    username: String,
    firstname: String,
    lastname: String,
    fullname: String,
    lang: String,
    userid: i32,
    siteurl: String,
    userpictureurl: String,
    functions: Vec<Function>,
    downloadfiles: Option<i32>,
    uploadfiles: Option<i32>,
    release: Option<String>,
    version: Option<String>,
    mobilecssurl: Option<String>,
    advancedfeatures: Option<Vec<AdvancedFeature>>,
    usercanmanageownfiles: Option<bool>,
    userquota: Option<i32>,
    usermaxuploadfilesize: Option<i32>,
    userhomepage: Option<i32>,
    userhomepageurl: Option<String>,
    userprivateaccesskey: Option<String>,
    siteid: Option<i32>,
    sitecalendartype: Option<String>,
    usercalendartype: Option<String>,
    userissiteadmin: Option<bool>,
    theme: Option<String>,
    limitconcurrentlogins: Option<i32>,
    usersessionscount: Option<i32>,
    policyagreed: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Function {
    name: String,
    version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdvancedFeature {
    name: String,
    value: i32,
}
