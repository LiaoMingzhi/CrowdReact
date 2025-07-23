import { createRouter, createWebHistory } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { ElMessage } from 'element-plus'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/Home.vue')
    },
    {
      path: '/betting',
      name: 'betting',
      component: () => import('@/views/Betting.vue'),
      meta: { allowUnauth: true }
    },
    {
      path: '/bet-history',
      name: 'bet-history',
      component: () => import('@/views/BetHistory.vue'),
      meta: { requiresAuth: true }
    },
    {
      path: '/agent',
      name: 'agent',
      component: () => import('@/views/Agent.vue'),
      meta: { requiresAuth: true }
    },
    {
      path: '/commission',
      name: 'commission',
      component: () => import('@/views/Commission.vue'),
      meta: { requiresAuth: true }
    }
  ]
})

router.beforeEach((to, from, next) => {
  const userStore = useUserStore()
  
  if (to.meta.requiresAuth && !userStore.isLoggedIn) {
    next()
  } else {
    next()
  }
})

export default router 