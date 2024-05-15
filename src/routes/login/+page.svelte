<script>
    export let form;

    const login = async ({ request, locals }) => {
        const body = await request.formData();
        const email = String(body.get("email"));
        const password = String(body.get("password"));

        const login = await tryLogin(locals.pb, email, password);
        if (login.res == Response.error) {
            await tryRegister(locals.pb, email, password);

            const login = await tryLogin(locals.pb, email, password);

            if (login.res == Response.error) {
                throw error(500, "Something went wrong");
            }

            const ctype = await tryChatTypes(locals.pb, login.user_id);

            if (ctype == Response.error) {
                throw error(500, "Something went wrong");
            }
        }

        throw redirect(303, "/");
    };

    const tryLogin = async (email, password) => {
        let user_id = "";
        try {
            user_id = (
                await pb.collection("users").authWithPassword(email, password)
            ).record.id;
        } catch (err) {
            console.log(err);
            return { res: Response.error, user_id };
        }
        console.log("successfully logged in", user_id);
        return { res: Response.ok, user_id };
    };
</script>

<main>
    <div class="outer-panel">
        <div class="panel">
            <div class="login-wrapper">
                <form on:submit={login} method="POST">
                    <div class="form-item">
                        <label for="email">Email</label>
                        <input type="email" name="email" id="email" />
                    </div>
                    <div class="form-item">
                        <label for="password">Password</label>
                        <input type="password" name="password" id="password" />
                    </div>
                    <div class="form-item">
                        <button class="form-button" type="submit"
                            >Login/Register</button
                        >
                    </div>
                    {#if form != null && form.error}
                        <div class="form-item">
                            <p class="error">{form.message}</p>
                        </div>
                    {/if}
                </form>
            </div>
        </div>
    </div>
</main>

<style lang="scss">
    @import "./src/sass/variables.scss";

    :root {
        box-sizing: border-box;
    }

    .panel {
        height: auto;
        width: 50%;
    }

    form {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;

        padding: $pad00;

        width: 100%;
    }

    .form-button {
        background-color: $base06;
        color: $base01;
        padding: $pad01;
        border: $bWidth solid $base04;
        border-radius: $bRad;
    }

    .form-item {
        display: flex;
        flex-direction: column;

        width: 100%;

        padding: $pad00;
        padding-bottom: 0;
    }

    .form-item label {
        padding: $pad01;
        padding-top: 0;
        padding-left: 0;
        padding-right: 0;
    }

    .form-item input {
        background-color: transparent;
        color: white;
        padding: $pad01;
        border: $bWidth solid $base04;
        border-radius: $bRad;
    }

    /* Change Autocomplete styles in Chrome*/
    input:-webkit-autofill,
    input:-webkit-autofill:hover,
    input:-webkit-autofill:focus {
        border: $bWidth solid $base04;
        border-radius: $bRad;
        -webkit-text-fill-color: $base0D;
        -webkit-box-shadow: 0 0 0px 1000px transparent inset;
        transition: background-color 10000s ease-in-out 0s;
        font-size: $fSizeSidebar;
    }

    /* Change Autocomplete styles in Chrome*/
    input:-webkit-autofill:hover {
        border: $bWidth solid $base05;
    }

    input:-webkit-autofill:focus {
        border: $bWidth solid $base06;
    }

    button:hover {
        cursor: grab;
        background-color: $base07;
    }

    button:active {
        cursor: grabbing;
        background-color: $base04;
    }

    .login-wrapper {
        display: flex;
        position: relative;

        flex-direction: column;
        flex-wrap: nowrap;
        justify-content: space-between;
        align-items: normal;
        align-content: normal;

        width: 100%;
        height: 100%;

        overflow: hidden;
    }
</style>
