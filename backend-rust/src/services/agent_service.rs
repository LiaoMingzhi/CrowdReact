// src/services/agent_service.rs
use crate::models::agent_model::{self, Entity as Agent, Model as AgentModel, ActiveModel};
use rand::seq::SliceRandom;
use sea_orm::{
    ActiveModelTrait, 
    DatabaseConnection, 
    DbErr, 
    EntityTrait, 
    QueryFilter,
    ColumnTrait,
    Set
};
use std::sync::Arc;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct AgentService {
    db: Arc<DatabaseConnection>,
}

impl AgentService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    // pub async fn create_agent(
    //     &self,
    //     user_address: String,
    //     level_agent: String,
    //     superior_address: Option<String>,
    // ) -> Result<AgentModel, DbErr> {
    //     // Validate level_agent value
    //     if !["one", "two", "common", "not_agent"].contains(&level_agent.as_str()) {
    //         return Err(DbErr::Custom("Invalid agent level".to_string()));
    //     }
    // 
    //     // If superior_address is provided, verify it exists
    //     if let Some(ref superior) = superior_address {
    //         let exists = Agent::find()
    //             .filter(agent_model::Column::UserAddress.eq(superior.clone()))
    //             .one(&*self.db)
    //             .await?
    //             .is_some();
    // 
    //         if !exists {
    //             return Err(DbErr::Custom("Superior agent not found".to_string()));
    //         }
    //     }
    // 
    //     let agent = agent_model::ActiveModel {
    //         user_address: Set(user_address),
    //         level_agent: Set(level_agent),
    //         superior_address: Set(superior_address),
    //         is_frozen: Set(false),
    //         ..Default::default()
    //     };
    // 
    //     agent.insert(&*self.db).await
    // }

    // pub async fn create_agent_for_user(
    //     &self,
    //     user_address: String,
    // ) -> Result<AgentModel, DbErr> {
    //     self.create_agent(
    //         user_address,
    //         "not_agent".to_string(),
    //         None,
    //     ).await
    // }

    

    // 获取所有一代理人
    pub async fn get_all_level_one_agents(&self) -> Result<Vec<AgentModel>, DbErr> {
        Agent::find()
            .filter(agent_model::Column::LevelAgent.eq("one"))
            .all(&*self.db)
            .await
    }

    // 冻结代理人
    pub async fn freeze_agent(&self, account_address: String) -> Result<(), DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(account_address))
            .one(&*self.db)
            .await?;

        if let Some(agent) = agent {
            let mut active_agent: agent_model::ActiveModel = agent.into();
            active_agent.is_frozen = Set(true);
            active_agent.update(&*self.db).await?;
        }

        Ok(())
    }

    // 获取随机一级代理
    pub async fn get_random_level_one_agent(&self) -> Result<String, DbErr> {
        let level_one_agents = Agent::find()
            .filter(agent_model::Column::LevelAgent.eq("one"))
            .all(&*self.db)
            .await?;

        if level_one_agents.is_empty() {
            return Err(DbErr::Custom("No level one agents available".to_string()));
        }

        let random_agent = level_one_agents
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| DbErr::Custom("Failed to select random agent".to_string()))?;

        Ok(random_agent.user_address.clone())
    }

    // 获取指定等级的代理人名单
    pub async fn get_agents_by_level(&self, level: &str) -> Result<Vec<AgentModel>, DbErr> {
        Agent::find()
            .filter(agent_model::Column::LevelAgent.eq(level))
            .all(&*self.db)
            .await
    }

    // // 增加一级代理人
    // pub async fn create_level_one_agent(
    //     &self,
    //     account_address: String,
    //     upper_agent_address: Option<String>,
    //     upper_agent_role: Option<String>,
    // ) -> Result<AgentModel, DbErr> {
    //     let agent = agent_model::ActiveModel {
    //         account_address: Set(account_address),
    //         role_name: Set("level_one".to_string()),
    //         upper_agent_address: Set(upper_agent_address),
    //         upper_agent_role: Set(upper_agent_role),
    //         level_two_agent: Set(None),
    //         level_one_agent: Set(None),
    //         level_common_agent: Set(None),
    //         is_frozen: Set(false),
    //     };
    // 
    //     agent.insert(&*self.db).await
    // }

    // 获取所有二级代理人
    pub async fn get_all_level_two_agents(&self) -> Result<Vec<AgentModel>, DbErr> {
        Agent::find()
            .filter(agent_model::Column::LevelAgent.eq("two"))
            .all(&*self.db)
            .await
    }

    // // 增加二级代理人
    // pub async fn create_level_two_agent(
    //     &self,
    //     account_address: String,
    //     upper_agent_address: String,
    //     upper_agent_role: String,
    // ) -> Result<AgentModel, DbErr> {
    //     let agent = agent_model::ActiveModel {
    //         account_address: Set(account_address),
    //         role_name: Set("level_two".to_string()),
    //         upper_agent_address: Set(Some(upper_agent_address)),
    //         upper_agent_role: Set(Some(upper_agent_role)),
    //         level_two_agent: Set(None),
    //         level_one_agent: Set(None),
    //         level_common_agent: Set(None),
    //         is_frozen: Set(false),
    //     };
    // 
    //     agent.insert(&*self.db).await
    // }

    // 获取所有普通代理人
    pub async fn get_all_normal_agents(&self) -> Result<Vec<AgentModel>, DbErr> {
        Agent::find()
            .filter(agent_model::Column::LevelAgent.eq("common"))
            .all(&*self.db)
            .await
    }

    // 增加普通代理人
    // pub async fn create_normal_agent(
    //     &self,
    //     account_address: String,
    //     upper_agent_address: Option<String>,
    //     upper_agent_role: Option<String>,
    // ) -> Result<AgentModel, DbErr> {
    //     let agent = agent_model::ActiveModel {
    //         account_address: Set(account_address),
    //         role_name: Set("normal".to_string()),
    //         upper_agent_address: Set(upper_agent_address),
    //         upper_agent_role: Set(upper_agent_role),
    //         level_two_agent: Set(None),
    //         level_one_agent: Set(None),
    //         level_common_agent: Set(None),
    //         is_frozen: Set(false),
    //     };
    // 
    //     agent.insert(&*self.db).await
    // }

    // 获取一级代理和二级代理的对应关系
    // pub async fn get_one_two_relation(&self) -> Result<Vec<(String, String)>, DbErr> {
    //     let relations = Agent::find()
    //         .filter(agent_model::Column::RoleName.eq("level_one"))
    //         .all(&*self.db)
    //         .await?;
    // 
    //     let mut result = Vec::new();
    //     for agent in relations {
    //         if let Some(level_two_agent) = agent.level_two_agent {
    //             result.push((agent.account_address.clone(), level_two_agent));
    //         }
    //     }
    // 
    //     Ok(result)
    // }

    // 增加一级代理和二级代理的对应关系
    // pub async fn add_one_two_relation(
    //     &self,
    //     one_agent_id: String,
    //     two_agent_id: String,
    // ) -> Result<(), DbErr> {
    //     // 创建新的代理关系
    //     let relation = RelationActiveModel {
    //         level_one_agent_id: Set(Some(one_agent_id)),
    //         level_two_agent_id: Set(Some(two_agent_id)),
    //         normal_agent_id: Set(None), // 这里可以根据需要设置为 None
    //         ..Default::default()                    // 其他字段使用默认值
    //     };
    // 
    //     // 插入关系到数据库
    //     relation.insert(&*self.db).await?;
    //     Ok(())
    // }

    // 获取二级代理和普通代理的对应关系
    // pub async fn get_two_common_relation(&self) -> Result<Vec<(String, String)>, DbErr> {
    //     let relations = Agent::find()
    //         .filter(agent_model::Column::RoleName.eq("level_two"))
    //         .all(&*self.db)
    //         .await?;
    // 
    //     let mut result = Vec::new();
    //     for agent in relations {
    //         if let Some(common_agent) = agent.level_one_agent {
    //             result.push((agent.account_address.clone(), common_agent));
    //         }
    //     }
    // 
    //     Ok(result)
    // }

    // 增加二级代理和普通代理的对应关系
    // pub async fn add_two_common_relation(
    //     &self,
    //     two_agent_id: String,
    //     common_agent_id: String,
    // ) -> Result<(), DbErr> {
    //     // 创建新的代理关系
    //     let relation = agent_relation_model::ActiveModel {
    //         id: Set(Default::default()),
    //         level_one_agent_id: Set(None),
    //         level_two_agent_id: Set(Some(two_agent_id)),
    //         normal_agent_id: Set(Some(common_agent_id)),
    //     };
    // 
    //     // 插入关系到数据库
    //     relation.insert(&*self.db).await?;
    //     Ok(())
    // }

    // 根据二级代理人地址获取挂载的一级代理人地址
    pub async fn get_level_one_agent_by_secondary(
        &self,
        secondary_agent_address: String,
    ) -> Result<Option<String>, DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(secondary_agent_address))
            .one(&*self.db)
            .await?;

        Ok(agent.and_then(|a| a.superior_address))
    }

    // 根据普通代理人地址获取挂载的一级代理人和二级代理人地址
    // pub async fn get_agents_by_common(
    //     &self,
    //     common_agent_address: String,
    // ) -> Result<(Option<String>, Option<String>), DbErr> {
    //     let agent = Agent::find()
    //         .filter(agent_model::Column::UserAddress.eq(common_agent_address))
    //         .one(&*self.db)
    //         .await?;
    // 
    //     if let Some(a) = agent {
    //         Ok((a.level_one_agent, a.level_two_agent))
    //     } else {
    //         Ok((None, None))
    //     }
    // }

    // 根据用户地址获取挂载的普通代理人、二级代理人和一级代理人地址
    // pub async fn get_agents_by_user(
    //     &self,
    //     user_address: String,
    // ) -> Result<(Option<String>, Option<String>, Option<String>), DbErr> {
    //     let agent = Agent::find()
    //         .filter(agent_model::Column::AccountAddress.eq(user_address))
    //         .one(&*self.db)
    //         .await?;
    // 
    //     if let Some(a) = agent {
    //         Ok((
    //             Some(a.account_address),
    //             a.level_two_agent,
    //             a.level_one_agent,
    //         ))
    //     } else {
    //         Ok((None, None, None))
    //     }
    // }

    // 获取用户角色和上级代理人信息
    pub async fn get_user_role_and_supervisors(
        &self,
        user_address: String,
    ) -> Result<(Option<String>, Option<String>), DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(user_address))
            .one(&*self.db)
            .await?;

        if let Some(a) = agent {
            Ok((Some(a.level_agent), a.superior_address))
        } else {
            Ok((None, None))
        }
    }

    // 获取代理信息
    pub async fn get_agent_info(
        &self,
        agent_address: String,
    ) -> Result<Option<(String, String)>, DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(agent_address))
            .one(&*self.db)
            .await?;

        if let Some(a) = agent {
            Ok(Some((a.level_agent, a.user_address)))
        } else {
            Ok(None)
        }
    }

    // 随机获取一个普通代理人
    pub async fn get_random_common_agent(&self) -> Result<String, DbErr> {
        // 查询所有通代理人
        let common_agents = Agent::find()
            .filter(agent_model::Column::LevelAgent.eq("common"))
            .all(&*self.db)
            .await?;

        // 如果没有找到普通代理人，返回错误
        if common_agents.is_empty() {
            return Err(DbErr::Custom("No common agents found".to_string()));
        }

        // 随机选择一个代理人
        let random_agent = common_agents
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| DbErr::Custom("Failed to select random agent".to_string()))?;

        // 返回选中代理人的地址
        Ok(random_agent.user_address.clone())
    }

    // 验证是否为普通代理人
    pub async fn is_common_agent(&self, address: &str) -> Result<bool, DbErr> {
        // 查询指定地址的代理人信息
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(address))
            .filter(agent_model::Column::LevelAgent.eq("common"))
            .one(&*self.db)
            .await?;

        // 如果找到记录且角色为普通代理人，返回 true
        Ok(agent.is_some())
    }

    // 根据普通代理人地址获取对应的二级代理人
    pub async fn get_level_two_agent_by_common(
        &self,
        common_agent_address: &str,
    ) -> Result<Option<String>, DbErr> {
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(common_agent_address))
            .filter(agent_model::Column::LevelAgent.eq("common"))
            .one(&*self.db)
            .await?;

        // 从代理人记录中获取二级代理人地址
        Ok(agent.and_then(|a| a.superior_address))
    }

    // 根据普通代理人地址获取对应的一级代理人
    pub async fn get_level_one_agent_by_common(
        &self,
        common_agent_address: &str,
    ) -> Result<Option<String>, DbErr> {
        // 先获取二级代理人
        if let Some(level_two_agent) = self
            .get_level_two_agent_by_common(common_agent_address)
            .await?
        {
            // 如果有二级代理人，通过二级代理人获取一级代理人
            self.get_level_one_agent_by_secondary(level_two_agent).await
        } else {
            // 如果没有二级代理人，直接查询普通代理人关联的一级代理人
            let agent = Agent::find()
                .filter(agent_model::Column::UserAddress.eq(common_agent_address))
                .filter(agent_model::Column::LevelAgent.eq("common"))
                .one(&*self.db)
                .await?;

            Ok(agent.and_then(|a| a.superior_address))
        }
    }

    pub async fn freeze_level_one_agent(&self, account_address: String) -> Result<(), DbErr> {
        // Find the agent by address
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(account_address.clone()))
            .one(&*self.db)
            .await?;

        match agent {
            Some(agent) => {
                let mut active_agent: agent_model::ActiveModel = agent.into();
                // Update to level one agent and freeze
                active_agent.level_agent = Set("one".to_string());
                active_agent.is_frozen = Set(true);
                // updated_at will be automatically set by the database trigger
                active_agent.update(&*self.db).await?;
                Ok(())
            }
            None => Err(DbErr::Custom(format!(
                "Agent not found with address: {}",
                account_address
            ))),
        }
    }

    pub async fn freeze_level_two_agent(&self, account_address: String, superior_address: String) -> Result<(), DbErr> {
        // Find the agent by address
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(account_address.clone()))
            .one(&*self.db)
            .await?;

        match agent {
            Some(agent) => {
                let mut active_agent: agent_model::ActiveModel = agent.into();
                // Update to level two agent and freeze
                active_agent.level_agent = Set("two".to_string());
                active_agent.is_frozen = Set(true);
                
                // Only set superior_address if it's not empty
                if !superior_address.is_empty() {
                    active_agent.superior_address = Set(Some(superior_address));
                }
                
                // updated_at will be automatically set by the database trigger
                active_agent.update(&*self.db).await?;
                Ok(())
            }
            None => Err(DbErr::Custom(format!(
                "Agent not found with address: {}",
                account_address
            ))),
        }
    }

    pub async fn freeze_level_common_agent(&self, account_address: String, superior_address: String) -> Result<(), DbErr> {
        // Find the agent by address
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(account_address.clone()))
            .one(&*self.db)
            .await?;

        match agent {
            Some(agent) => {
                let mut active_agent: agent_model::ActiveModel = agent.into();
                // Update to level two agent and freeze
                active_agent.level_agent = Set("common".to_string());
                active_agent.is_frozen = Set(true);
                // Only set superior_address if it's not empty  
                if !superior_address.is_empty() {   
                    active_agent.superior_address = Set(Some(superior_address));
                }
                
                // updated_at will be automatically set by the database trigger
                active_agent.update(&*self.db).await?;
                Ok(())
            }
            None => Err(DbErr::Custom(format!(
                "Agent not found with address: {}",
                account_address
            ))),
        }
    }

    pub async fn freeze_level_agent(&self, account_address: String, level_agent: String) -> Result<(), DbErr> {
        // Find the agent by address
        let agent = Agent::find()
            .filter(agent_model::Column::UserAddress.eq(account_address.clone()))
            .one(&*self.db)
            .await?;

        match agent {
            Some(agent) => {
                let mut active_agent: agent_model::ActiveModel = agent.into();
                // Update to level two agent and freeze
                active_agent.level_agent = Set(level_agent.to_string());
                active_agent.is_frozen = Set(true);
                // updated_at will be automatically set by the database trigger
                active_agent.update(&*self.db).await?;
                Ok(())
            }
            None => Err(DbErr::Custom(format!(
                "Agent not found with address: {}",
                account_address
            ))),
        }
    }

    pub async fn create_agent(&self, user_address: String) -> Result<AgentModel, DbErr> {
        let agent = agent_model::ActiveModel {
            user_address: Set(user_address),
            level_agent: Set("not_agent".to_string()),
            superior_address: Set(None),
            is_frozen: Set(false),
            ..Default::default()
        };

        agent.insert(&*self.db).await
    }

    /// Checks if the given address is a level one agent
    pub async fn is_level_one_agent(&self, address: &str) -> Result<bool, DbErr> {
        let agent = agent_model::Entity::find()
            .filter(agent_model::Column::UserAddress.eq(address))
            .filter(agent_model::Column::LevelAgent.eq("one"))
            .filter(agent_model::Column::IsFrozen.eq(true)) 
            .one(&*self.db)
            .await?;
        
        Ok(agent.is_some())
    }

    /// Checks if a given address is a level two agent
    pub async fn is_level_two_agent(&self, address: &str) -> Result<bool, DbErr> {
        let agent = agent_model::Entity::find()
            .filter(agent_model::Column::UserAddress.eq(address))
            .filter(agent_model::Column::LevelAgent.eq("two"))
            .filter(agent_model::Column::IsFrozen.eq(true))  // Only consider frozen (confirmed) agents
            .one(&*self.db)
            .await?;
        
        Ok(agent.is_some())
    }

    /// Gets the level one agent address for a given level two agent
    /// Returns None if the level two agent doesn't exist or doesn't have a superior
    pub async fn get_level_one_agent_for_level_two(&self, level_two_address: &str) -> Result<Option<String>, DbErr> {
        // Find the level two agent and their superior (level one agent)
        let level_two_agent = agent_model::Entity::find()
            .filter(agent_model::Column::UserAddress.eq(level_two_address))
            .filter(agent_model::Column::LevelAgent.eq("two"))
            .filter(agent_model::Column::IsFrozen.eq(true))
            .one(&*self.db)
            .await?;

        // If level two agent exists and has a superior, verify the superior is a level one agent
        if let Some(agent) = level_two_agent {
            if let Some(superior_address) = agent.superior_address {
                // Verify the superior is a level one agent
                let is_level_one = agent_model::Entity::find()
                    .filter(agent_model::Column::UserAddress.eq(&superior_address))
                    .filter(agent_model::Column::LevelAgent.eq("one"))
                    .filter(agent_model::Column::IsFrozen.eq(true))
                    .one(&*self.db)
                    .await?;

                if is_level_one.is_some() {
                    return Ok(Some(superior_address));
                }
            }
        }

        Ok(None)
    }

    /// Gets the level two agent address for a given common agent
    /// Returns None if the common agent doesn't exist or doesn't have a superior
    pub async fn get_level_two_agent_for_common(&self, common_agent_address: &str) -> Result<Option<String>, DbErr> {
        // Find the common agent and their superior (level two agent)
        let common_agent = agent_model::Entity::find()
            .filter(agent_model::Column::UserAddress.eq(common_agent_address))
            .filter(agent_model::Column::LevelAgent.eq("common"))
            .filter(agent_model::Column::IsFrozen.eq(true))
            .one(&*self.db)
            .await?;

        // If common agent exists and has a superior, verify the superior is a level two agent
        if let Some(agent) = common_agent {
            if let Some(superior_address) = agent.superior_address {
                // Verify the superior is a level two agent
                let is_level_two = agent_model::Entity::find()
                    .filter(agent_model::Column::UserAddress.eq(&superior_address))
                    .filter(agent_model::Column::LevelAgent.eq("two"))
                    .filter(agent_model::Column::IsFrozen.eq(true))
                    .one(&*self.db)
                    .await?;

                if is_level_two.is_some() {
                    return Ok(Some(superior_address));
                }
            }
        }

        Ok(None)
    }
}

