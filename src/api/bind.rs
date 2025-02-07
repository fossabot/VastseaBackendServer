use bson::doc;
use mongodb::Collection;
use crate::MONGODB;
use crate::user::UserInfo;

pub async fn bind_qq(uuid: String, qq: i64) -> Result<(), ()> {
    let collection: Collection<UserInfo> = MONGODB.collection("users");
    let user = collection.find_one(doc! {"_id": uuid}, None).await.unwrap();


    if user.is_none() || user.clone().unwrap().bind_qq != None {
        return Err(());
    }
    let user = user.unwrap();

    let collection: Collection<UserInfo> = MONGODB.collection("users");
    collection.update_many(doc! {"_id": user._id.clone()}, doc! {"$set": {"bind_qq": Some(qq)}}, None).await.unwrap();

    Ok(())
}