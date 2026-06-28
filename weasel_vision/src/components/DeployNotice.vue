<template>
  <div class="deploy-notice">
    <span class="notice-icon">⚠️</span>
    <span class="notice-text">本页配置修改保存后，需要重新部署才能生效</span>
  </div>
</template>

<style scoped>
.deploy-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--color-warning-light);
  border: 1px solid var(--color-warning);
  border-radius: 6px;
  margin-bottom: 16px;
}

.notice-icon {
  font-size: 16px;
}

.notice-text {
  font-size: 13px;
  color: var(--color-warning-dark);
  font-weight: 500;
}
</style>
