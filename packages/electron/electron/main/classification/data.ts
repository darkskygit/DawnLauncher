import { deleteAssociateFolderWatcher } from '.'
import { Classification, ClassificationData } from '../../../types/classification'
import { newClassification, newClassificationData } from '../../../commons/utils/common'
import { deleteByClassificationId } from '../item/data'
import { getDataSource, getDataSqlite3 } from '../../commons/betterSqlite3'

const dataSource = getDataSource()

// 获取数据库
let db = getDataSqlite3()

// 分类表名
let classificationTableName = 'classification'

/**
 * 分类
 */
function getClassification(row: any): Classification {
	return newClassification({
		id: row.id,
		parentId: row.parentId,
		name: row.name,
		type: row.type,
		data: newClassificationData(JSON.parse(row.data)),
		shortcutKey: row.shortcutKey,
		globalShortcutKey: [1, true].includes(row.globalShortcutKey),
		order: row.order,
	})
}

/**
 * 初始化
 */
export function init() {
	if (dataSource.getClassificationCount() === 0) {
		// 新增分类
		add(null, global.language.newClassificationName, null, false)
	}
}

/**
 * 列表
 */
export function list(parentId: number | null = null) {
	const list = dataSource.getClassification(parentId ? parentId : null)

	return list.map((row) => {
		return getClassification(row)
	})
}

/**
 * 添加
 */
export function add(
	parentId: number | null,
	name: string,
	shortcutKey: string | null,
	globalShortcutKey: boolean,
	data: ClassificationData = newClassificationData({}),
	type: number = 0
): Classification | null {
	return getClassification(dataSource.insertClassification(parentId, name, shortcutKey, globalShortcutKey, JSON.stringify(data), type))
}

/**
 * 更新
 */
export function update(classification: Classification) {
	console.log(classification.globalShortcutKey)
	return dataSource.updateClassification(
		classification.id,
		classification.name,
		classification.shortcutKey,
		classification.globalShortcutKey,
		JSON.stringify(classification.data),
		classification.type
	)
}

/**
 * 更新数据
 */
export function updateData(id: number, data: ClassificationData) {
	return dataSource.updateClassificationData(id, JSON.stringify(data)) > 0
}

/**
 * 根据ID查询
 */
export function selectById(id: number): Classification | null {
	return getClassification(dataSource.getClassificationById(id))
}

/**
 * 删除
 */
export function del(id: number) {
	// 查询数据
	let classifictaion = selectById(id)
	if (classifictaion) {
		// 查询有无子分类
		let childList = list(classifictaion.id)
		// SQL
		let sql = `DELETE FROM ${classificationTableName} WHERE id = ? or parent_id = ?`
		// 运行
		let res = db.prepare(sql).run(id, id).changes > 0
		if (res) {
			// 更新序号
			dataSource.reorderClassification(classifictaion.parentId)
			// 删除分类下所有项目
			deleteByClassificationId(id)
			// 删除子分类下所有项目
			for (const child of childList) {
				deleteByClassificationId(child.id)
				if (child.type === 1) {
					// 删除关联文件夹
					deleteAssociateFolderWatcher(child.id)
				}
			}
			if (classifictaion.type === 1) {
				// 删除关联文件夹
				deleteAssociateFolderWatcher(classifictaion.id)
			}
			return true
		} else {
			return false
		}
	} else {
		return false
	}
}

/**
 * 排序
 */
export function updateOrder(fromId: number, toId: number | null, parentId: number | null) {
	// 查询来源分类
	let fromClassification = selectById(fromId)
	if (fromClassification) {
		// 新序号
		let newOrder = 1
		// 如果目标ID不为空获取项目并获取序号
		if (toId) {
			let toClassification = selectById(toId)
			if (toClassification) {
				newOrder = toClassification.order
			}
		} else {
			newOrder = dataSource.getClassificationMaxOrder(parentId) + 1
		}
		// SQL
		let sql = `UPDATE ${classificationTableName} SET \`order\` = ? WHERE id = ?`
		// 更新排序
		db.prepare(sql).run(newOrder, fromClassification.id)
		// 判断新序号和老序号之间的数据是+1还是-1
		if (newOrder > fromClassification.order) {
			// 新序号和老序号之间数据，序号-1
			let params = [fromClassification.order, newOrder, fromClassification.id]
			sql = `UPDATE ${classificationTableName} SET \`order\` = \`order\` - 1 WHERE \`order\` > ? AND \`order\` <= ? AND id != ?`
			if (parentId) {
				sql += ' AND parent_id = ?'
				params.push(parentId)
			} else {
				sql += ' AND parent_id is NULL'
			}
			db.prepare(sql).run(params)
		} else {
			// 新序号和老序号之间数据，序号+1
			let params = [newOrder, fromClassification.order, fromClassification.id]
			sql = `UPDATE ${classificationTableName} SET \`order\` = \`order\` + 1 WHERE \`order\` >= ? AND \`order\` < ? AND id != ?`
			if (parentId) {
				sql += ' AND parent_id = ?'
				params.push(parentId)
			} else {
				sql += ' AND parent_id is NULL'
			}
			db.prepare(sql).run(params)
		}
		return true
	}
	return false
}

/**
 * 更新图标
 */
export function updateIcon(id: number, icon: string | null) {
	// 查询分类
	let classification = selectById(id)
	if (classification) {
		// 更新图标
		return updateData(id, { ...classification.data, icon })
	}
	return false
}

/**
 * 是否有子分类
 */
export function hasChildClassification(id: number) {
	return dataSource.hasChildClassification(id)
}

/**
 * 批量更新固定分类
 */
export function batchUpdateFixed(id: number | null = null) {
	// 查询所有分类
	let classificationList = list()
	// 更新
	for (const classification of classificationList) {
		updateData(classification.id, {
			...classification.data,
			fixed: id === classification.id,
		})
	}
}

export { dataSource }
