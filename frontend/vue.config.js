module.exports = {
    devServer: {
        proxy: {
            '/api': {
                // TODO take config from an env var
                target: 'http://localhost:6789', // todo
                pathRewrite: {'^/api' : '/'}
            },
        }
    },
    lintOnSave: true
}
